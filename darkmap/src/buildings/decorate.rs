use std::time::Instant;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_mod_outline::{OutlineBundle, OutlineMeshExt, OutlineVolume};
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
        let exterior = exterior.points_cw().map(Coord::from).collect::<Vec<_>>();

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
        mesh.set_indices(Some(Indices::U32(indices)));

        let _res = mesh.generate_outline_normals();

        let mut cmds = commands.entity(entity);
        cmds.insert((
            meshes.add(mesh),
            materials.add(StandardMaterial {
                base_color: Color::rgb(0.2, 0.22, 0.25),
                ..Default::default()
            }),
            On::<Pointer<Over>>::listener_commands_mut(|_, cmds| {
                cmds.insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 2.,
                        colour: Color::rgb(1., 0., 0.),
                    },
                    ..default()
                });
            }),
            On::<Pointer<Out>>::listener_commands_mut(|_, cmds| {
                cmds.remove::<OutlineBundle>();
            }),
        ));

        if height < 12. {
            cmds.insert(ViewDistance(1000.));
        }
    }
}
