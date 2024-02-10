use std::f32::consts::FRAC_PI_2;

use anyhow::Context;
use bevy::{
    pbr::StandardMaterial,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use geo::{Centroid, CoordsIter, HaversineBearing, HaversineDistance, LineString};
use itertools::Itertools;
use serde_json::json;

use crate::{
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
    overpass::{Element, Tags},
};

#[derive(Default)]
pub struct RoadsPlugin;

impl Plugin for RoadsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingPlugin::<Road>::new())
            .add_systems(Update, decorate_road);
    }
}

#[derive(Component)]
pub struct Road {
    pub geometry: LineString,
}

impl LoadType for Road {
    type Bundle = impl Bundle;

    async fn load(req: LoadRequest) -> anyhow::Result<Vec<Self::Bundle>> {
        let template = include_str!("../../assets/queries/roads.ovp");

        let query = handlebars::Handlebars::new()
            .render_template(template, &json!({ "bbox": req.bbox() }))
            .context("Failed to render query")?;

        let res = crate::overpass::load(&query)
            .await
            .context("Failed to load roads")?;

        Ok(res
            .elements
            .into_iter()
            .flat_map(|elem| {
                if let Element::Way(way) = elem {
                    Some((
                        Self { geometry: way.geometry.into() },
                        way.tags,
                        WorldPosition(way.bounds.unwrap().centroid()),
                        DecorateRequest,
                    ))
                } else {
                    None
                }
            })
            .collect())
    }
}

fn decorate_road(
    query: Query<(Entity, &Road, &Tags, &WorldPosition), With<DecorateRequest>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, road, tags, pos) in query.iter().take(100) {
        commands.entity(entity).remove::<DecorateRequest>();

        let origin = pos.0;

        let layer = tags
            .0
            .get("layer")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0) as f32
            * 0.1;

        let level = tags
            .0
            .get("level")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0) as f32
            * 0.;

        let bias = match tags.0.get("highway").map(|s| s.as_str()) {
            Some("motorway") => 0.5,
            Some("trunk") => 0.4,
            Some("primary") => 0.3,
            Some("secondary") => 0.2,
            Some("tertiary") => 0.1,
            Some("residential") => 0.,
            Some("service") => -0.1,
            Some("unclassified") => -0.2,
            Some("cycleway") => -0.3,
            Some("footway") => -0.4,
            _ => 0.,
        } * 0.01;

        let height = if tags.0.get("bridge").is_some() {
            0.
        } else if tags.0.get("tunnel").is_some() {
            -0.
        } else {
            0.
        };

        // translate coords into meters
        let geometry = road
            .geometry
            .coords_iter()
            .map(|coord| {
                let distance = origin.haversine_distance(&coord.into());
                let bearing = origin.haversine_bearing(coord.into());

                let ang = bearing.to_radians();

                let x = (distance * ang.sin()) as f32;
                let y = (distance * -ang.cos()) as f32;

                Vec3::new(x, height + level + layer + bias, y)
            })
            .collect::<Vec<_>>();

        let half_width: f32 = tags.road_width() / 2.;

        let mesh = if tags.0.get("area").is_some() {
            let vertices_2d = geometry.iter().flat_map(|v| [v.x, v.z]).collect::<Vec<_>>();

            let indices = earcutr::earcut(&vertices_2d, &[], 2)
                .expect("Failed to triangulate road")
                .into_iter()
                .map(|i| i as u32)
                .array_chunks()
                .flat_map(|[a, b, c]| [a, c, b])
                .collect::<Vec<_>>();

            let vertices = geometry.iter().map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>();

            let normals = (0..vertices.len())
                .map(|_| Vec3::Y.into())
                .collect::<Vec<[f32; 3]>>();

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.set_indices(Some(Indices::U32(indices)));
            mesh
        } else {
            let mut vertices = Vec::<[f32; 3]>::new();

            for (pos, (prev, this, next)) in geometry
                .iter()
                .take(1)
                .chain(&geometry)
                .chain(geometry.last())
                .tuple_windows()
                .with_position()
            {
                let angle = match pos {
                    itertools::Position::First => (*next - *this).normalize(),
                    itertools::Position::Middle => {
                        let prev_angle = (*this - *prev).normalize();
                        let next_angle = (*next - *this).normalize();
                        (prev_angle + next_angle).normalize()
                    }
                    itertools::Position::Last => (*this - *prev).normalize(),
                    itertools::Position::Only => break,
                };

                let left = Quat::from_rotation_y(FRAC_PI_2).mul_vec3(angle);
                let right = Quat::from_rotation_y(-FRAC_PI_2).mul_vec3(angle);

                vertices.push((*this + left * half_width).into());
                vertices.push((*this + right * half_width).into());
            }

            let normals = (0..vertices.len())
                .map(|_| Vec3::Y.into())
                .collect::<Vec<[f32; 3]>>();

            let indices = (0..vertices.len() as u32)
                .tuple_windows()
                .step_by(2)
                .flat_map(|(a, b, c, d)| [a, b, c, c, b, d])
                .collect::<Vec<_>>();

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.set_indices(Some(Indices::U32(indices)));
            mesh
        };

        let base_color = match tags.0.get("highway").map(|s| s.as_str()) {
            Some("motorway") => Color::rgb(0.9, 0.5, 0.1),
            Some("trunk") => Color::rgb(0.9, 0.4, 0.1),
            Some("primary") => Color::rgb(0.9, 0.3, 0.1),
            Some("secondary") => Color::rgb(0.7, 0.3, 0.3),
            Some("tertiary") => Color::rgb(0.6, 0.4, 0.4),
            Some("residential") => Color::rgb(0.5, 0.5, 0.5),
            Some("service") => Color::rgb(0.4, 0.6, 0.6),
            Some("unclassified") => Color::rgb(0.7, 0.7, 0.7),
            Some("cycleway") => Color::rgb(0.8, 0.6, 0.2),
            Some("footway") => Color::rgb(0.3, 0.1, 0.0),
            _ => Color::rgb(0.5, 0.5, 0.5),
        };

        let mut cmds = commands.entity(entity);
        cmds.insert((
            meshes.add(mesh),
            materials.add(StandardMaterial { base_color, ..Default::default() }),
        ));

        if let Some(name) = tags.name() {
            cmds.insert(Name::new(name.to_string()));
        }
    }
}
