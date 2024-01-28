use std::collections::HashMap;

use anyhow::Context;
use bevy::prelude::*;
use overpass::Element;
use serde_json::json;

use crate::{
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
};

#[derive(Default)]
pub struct PoiPlugin;

impl Plugin for PoiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingPlugin::<PointOfInterest>::new())
            .register_type::<PointOfInterest>()
            .add_systems(Update, decorate_poi);
    }
}

#[derive(Component, Reflect)]
pub struct PointOfInterest {
    pub tags: HashMap<String, String>,
}

impl LoadType for PointOfInterest {
    type Bundle = impl Bundle;

    async fn load(req: LoadRequest) -> anyhow::Result<Vec<Self::Bundle>> {
        let template = include_str!("../../assets/queries/poi.ovp");

        let query = handlebars::Handlebars::new()
            .render_template(template, &json!({ "bbox": req.bbox() }))
            .context("Failed to render query")?;

        let res = overpass::load(&query).await.context("Failed to load POI")?;

        Ok(res
            .elements
            .into_iter()
            .flat_map(|elem| {
                if let Element::Node(node) = elem {
                    Some((Self { tags: node.tags }, WorldPosition(node.point), DecorateRequest))
                } else {
                    None
                }
            })
            .collect())
    }
}

fn decorate_poi(
    query: Query<(Entity, &PointOfInterest), With<DecorateRequest>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, _poi) in &mut query.iter() {
        commands.entity(entity).remove::<DecorateRequest>().insert((
            meshes.add(shape::UVSphere { radius: 1., sectors: 8, stacks: 4 }.into()),
            materials.add(StandardMaterial {
                base_color: Color::rgb(1., 0., 0.),
                ..Default::default()
            }),
        ));
    }
}
