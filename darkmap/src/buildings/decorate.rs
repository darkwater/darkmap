use std::{f32::consts::FRAC_PI_4, time::Instant};

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_mod_outline::{OutlineBundle, OutlineVolume, ATTRIBUTE_OUTLINE_NORMAL};
use bevy_mod_picking::prelude::*;
use geo::{Coord, CoordsIter, HaversineBearing, HaversineDistance, LineString, MapCoords, Winding};
use itertools::Itertools;

use super::Building;
use crate::{
    common::{DecorateRequest, WorldPosition},
    viewport::view_distance::ViewDistance,
};

pub fn decorate_building(
    query: Query<(Entity, &Building, &WorldPosition), With<DecorateRequest>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let start = Instant::now();
    for (entity, building, pos) in &mut query.iter() {
        if start.elapsed().as_millis() > 2 {
            break;
        }

        commands.entity(entity).remove::<DecorateRequest>();

        let origin = pos.0;

        // translate coords into meters
        let geometry = building.geometry.map_coords(|coord| {
            let distance = origin.haversine_distance(&coord.into());
            let bearing = origin.haversine_bearing(coord.into());

            let ang = bearing.to_radians();

            Coord {
                x: (distance * ang.sin()) as f32,
                y: (distance * -ang.cos()) as f32,
            }
        });

        let height = building.tags.building_height().unwrap_or(10.);

        let exterior = geometry.exterior_coords_iter().collect::<LineString<f32>>();
        let mut exterior = exterior.points_cw().map(Coord::from).collect::<Vec<_>>();
        if exterior.first() == exterior.last() {
            exterior.pop();
        }

        // 2d vertices for earcutr
        let vertices = exterior.iter().flat_map(|c| [c.x, c.y]).collect::<Vec<_>>();

        // apparently this can happen
        if vertices.is_empty() {
            error!("Building has no vertices");
            continue;
        }

        // find the triangles
        let indices = earcutr::earcut(&vertices, &[], 2);
        let Ok(indices) = indices else {
            error!("Failed to triangulate building: {:?}", indices);
            continue;
        };

        let mut indices = indices
            .into_iter()
            .map(|i| i as u32)
            .array_chunks()
            .flat_map(|[a, b, c]| [a, c, b])
            .collect::<Vec<_>>();

        // 3d vertices for the roof
        let mut vertices = exterior
            .iter()
            .map(|c| [c.x, height, c.y])
            .collect::<Vec<_>>();

        let mut normals = exterior.iter().map(|_| [0., 1., 0.]).collect::<Vec<_>>();
        let mut colors = exterior
            .iter()
            .map(|_| [1., 1., 1., 1.])
            .collect::<Vec<_>>();

        let mut outline_normals = exterior
            .iter()
            .map(|c| Vec3::new(c.x, height, c.y))
            .circular_tuple_windows()
            .map(|(prev, this, next)| {
                let prev_angle = (this - prev).normalize();
                let next_angle = (next - this).normalize();
                let angle = (prev_angle + next_angle).normalize();

                Quat::from_axis_angle(angle, FRAC_PI_4)
                    .mul_vec3(Vec3::Y)
                    .to_array()
            })
            .collect::<Vec<_>>();
        outline_normals.rotate_right(1);

        // wall time
        // each wall needs its own set of vertices and normals
        let base = vertices.len() as u32;
        vertices.extend(exterior.iter().circular_tuple_windows().flat_map(|(a, b)| {
            [[a.x, 0., a.y], [a.x, height, a.y], [b.x, 0., b.y], [b.x, height, b.y]]
        }));
        normals.extend(
            exterior
                .iter()
                .map(|v| Vec3::new(v.x, 0., v.y))
                .circular_tuple_windows()
                .flat_map(|(a, b)| {
                    let normal = (b - a).normalize().cross(Vec3::Y);
                    [[normal.x, normal.y, normal.z]; 4]
                }),
        );
        outline_normals.extend_from_slice(
            &outline_normals
                .iter()
                .circular_tuple_windows()
                .flat_map(|(a, b)| {
                    [
                        [a[0], -a[1], a[2]], //
                        [a[0], a[1], a[2]],
                        [b[0], -b[1], b[2]],
                        [b[0], b[1], b[2]],
                    ]
                })
                .collect_vec(),
        );
        let wall_a = 0.5;
        colors.extend((0..exterior.len() * 4).map(|_| [wall_a, wall_a, wall_a, 1.]));
        indices.extend(
            (0..exterior.len())
                .flat_map(|i| {
                    let i = i as u32;
                    [[base + i * 4, base + i * 4 + 1, base + i * 4 + 2], [
                        base + i * 4 + 1,
                        base + i * 4 + 3,
                        base + i * 4 + 2,
                    ]]
                })
                .flat_map(|[a, b, c]| [a, c, b]),
        );

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_attribute(ATTRIBUTE_OUTLINE_NORMAL, outline_normals);
        mesh.set_indices(Some(Indices::U32(indices)));

        let mut cmds = commands.entity(entity);
        cmds.insert((
            meshes.add(mesh),
            materials.add(StandardMaterial {
                base_color: Color::rgb(0.2, 0.22, 0.25),
                ..Default::default()
            }),
            PickableBundle::default(),
            OutlineBundle {
                outline: OutlineVolume {
                    visible: false,
                    width: 2.,
                    colour: Color::rgb(1., 1., 1.),
                },
                ..default()
            },
            // On::<Pointer<Over>>::listener_commands_mut(|_, cmds| {
            //     cmds.add(|mut ent: EntityWorldMut| {
            //         let selected = ent.get::<PickSelection>().is_some_and(|ps| ps.is_selected);
            //         if !selected {
            //             ent.insert(OutlineBundle {
            //                 outline: OutlineVolume {
            //                     visible: true,
            //                     width: 2.,
            //                     colour: Color::rgb(1., 1., 1.),
            //                 },
            //                 ..default()
            //             });
            //         }
            //     });
            // }),
            // On::<Pointer<Select>>::listener_component_mut(|_, volume: &mut OutlineVolume| {
            //     println!("SELECTED");
            //     volume.colour = Color::rgb(0., 1., 0.);
            // }),
            // On::<Pointer<Out>>::listener_commands_mut(|_, cmds| {
            //     cmds.add(|mut ent: EntityWorldMut| {
            //         let only_hovering = ent
            //             .get::<OutlineVolume>()
            //             .is_some_and(|v| v.colour == Color::rgb(1., 1., 1.));

            //         if only_hovering {
            //             ent.remove::<OutlineBundle>();
            //         }
            //     });
            // }),
            // On::<Pointer<Deselect>>::listener_commands_mut(|_, cmds| {
            //     println!("DESELECTED");
            //     cmds.remove::<OutlineBundle>();
            // }),
        ));

        if let Some(name) = building.tags.name() {
            cmds.insert(Name::new(name.to_string()));
        }

        if height < 12. {
            cmds.insert(ViewDistance(1000.));
        }
    }
}
