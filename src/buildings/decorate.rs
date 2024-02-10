use std::f32::consts::FRAC_PI_4;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_mod_outline::ATTRIBUTE_OUTLINE_NORMAL;
use bevy_mod_picking::prelude::*;
use geo::{Coord, CoordsIter, HaversineBearing, HaversineDistance, LineString, MapCoords, Winding};
use itertools::Itertools;

use super::{material::Materials, Building};
use crate::{
    common::{DecorateRequest, WorldPosition},
    overpass::Tags,
    viewport::view_distance::ViewDistance,
};

pub fn decorate_building(
    query: Query<(Entity, &Building, &Tags, &WorldPosition), With<DecorateRequest>>,
    materials: Res<Materials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, building, tags, pos) in query.iter().take(100) {
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

        let height = tags.building_height().unwrap_or(10.);

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
            .map(|_| [0.5, 0.5, 0.5, 0.5])
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
        let wall_a = 0.3;
        colors.extend((0..exterior.len() * 2).flat_map(|_| {
            [[wall_a / 5., wall_a / 5., wall_a / 5., 1.], [wall_a, wall_a, wall_a, 1.]]
        }));
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

        let material = match tags.get("building").map(|s| s.as_str()) {
            Some(
                "boathouse" | "bungalow" | "cabin" | "static_caravan" | "terrace" | "apartments"
                | "house" | "residential" | "detached" | "semidetached_house",
            ) => materials.residential.clone(),
            Some("church" | "chapel" | "mosque" | "temple" | "religious") => {
                materials.religious.clone()
            }
            Some("farm_auxiliary" | "barn" | "greenhouse") => materials.agricultural.clone(),
            Some("school" | "university" | "kindergarten") => materials.school.clone(),
            Some("manufacture" | "industrial") => materials.industrial.clone(),
            Some("civic" | "public" | "stadium") => materials.civic.clone(),
            Some("commercial") => materials.commercial.clone(),
            Some("retail") => materials.retail.clone(),
            Some("outbuilding") => materials.outbuilding.clone(),
            Some("construction") => materials.construction.clone(),
            Some("service" | "fire_station") => materials.service.clone(),
            Some("farm") => materials.agricultural.clone(),
            Some("warehouse") => materials.warehouse.clone(),
            Some("office") => materials.office.clone(),
            Some("hospital") => materials.hospital.clone(),
            Some("hotel") => materials.hotel.clone(),
            Some("train_station" | "transportation") => materials.transportation.clone(),
            _ => materials.default.clone(),
        };

        let mut cmds = commands.entity(entity);
        cmds.insert((meshes.add(mesh), material, PickableBundle::default()));

        if let Some(name) = tags.name() {
            cmds.insert(Name::new(name.to_string()));
        }

        if height < 12. {
            cmds.insert(ViewDistance(800.));
        }
    }
}
