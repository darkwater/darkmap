use std::collections::HashMap;

use anyhow::Context;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use geo::{
    triangulate_spade::SpadeTriangulationConfig, Centroid, Coord, HaversineBearing,
    HaversineDistance, MapCoords, MultiPolygon, TriangulateSpade,
};
use overpass::Element;
use serde_json::json;

use crate::{
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
    viewport::view_distance::ViewDistance,
};

#[derive(Default)]
pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingPlugin::<Building>::new())
            .add_systems(Update, decorate_building);
    }
}

#[derive(Component)]
pub struct Building {
    pub tags: HashMap<String, String>,
    pub geometry: MultiPolygon,
}

impl LoadType for Building {
    type Bundle = impl Bundle;

    async fn load(req: LoadRequest) -> anyhow::Result<Vec<Self::Bundle>> {
        let template = include_str!("../../assets/queries/buildings.ovp");

        let query = handlebars::Handlebars::new()
            .render_template(template, &json!({ "bbox": req.bbox() }))
            .context("Failed to render query")?;

        let res = overpass::load(&query).await.context("Failed to load POI")?;

        Ok(res
            .elements
            .into_iter()
            .flat_map(|elem| {
                if let Element::Way(way) = elem {
                    way.polygon().map(|poly| {
                        (
                            Self {
                                tags: way.tags,
                                geometry: poly.into(),
                            },
                            WorldPosition(way.bounds.centroid()),
                            DecorateRequest,
                        )
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}

fn decorate_building(
    query: Query<(Entity, &Building, &WorldPosition), With<DecorateRequest>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, building, pos) in &mut query.iter() {
        commands.entity(entity).remove::<DecorateRequest>();

        let origin = pos.0;

        let geometry = building.geometry.map_coords(|coord| {
            let distance = origin.haversine_distance(&coord.into());
            let bearing = origin.haversine_bearing(coord.into());

            let ang = bearing.to_radians();

            Coord {
                x: (distance * ang.sin()) as f32,
                y: (distance * -ang.cos()) as f32,
            }
        });

        let tris = geometry.constrained_triangulation(SpadeTriangulationConfig::default());

        let Ok(tris) = tris else {
            error!("Failed to triangulate building: {:?}", tris);
            continue;
        };

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for tri in tris {
            for c in [tri.0, tri.2, tri.1] {
                let c = [c.x, 0., c.y];

                if let Some(idx) = vertices.iter().position(|v| *v == c) {
                    indices.push(idx as u32);
                } else {
                    indices.push(vertices.len() as u32);
                    vertices.push(c);
                }
            }
        }

        if vertices.is_empty() {
            error!("Building has no vertices");
            continue;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.set_indices(Some(Indices::U32(indices)));

        commands.entity(entity).insert((
            meshes.add(mesh),
            materials.add(StandardMaterial {
                base_color: Color::rgb(1., 1., 0.),
                ..Default::default()
            }),
            ViewDistance(1000.),
        ));
    }
}
