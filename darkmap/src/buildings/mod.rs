mod decorate;

use anyhow::Context;
use bevy::prelude::*;
use geo::{Centroid, MultiPolygon};
use overpass::{Element, Tags};
use serde_json::json;

use crate::{
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
};

#[derive(Default)]
pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingPlugin::<Building>::new())
            .add_systems(Update, decorate::decorate_building);
    }
}

#[derive(Component)]
pub struct Building {
    pub tags: Tags,
    pub geometry: MultiPolygon,
}

impl LoadType for Building {
    type Bundle = impl Bundle;

    async fn load(req: LoadRequest) -> anyhow::Result<Vec<Self::Bundle>> {
        let template = include_str!("../../assets/queries/buildings.ovp");

        let query = handlebars::Handlebars::new()
            .render_template(template, &json!({ "bbox": req.bbox() }))
            .context("Failed to render query")?;

        let res = overpass::load(&query)
            .await
            .context("Failed to load buildings")?;

        Ok(res
            .elements
            .into_iter()
            .flat_map(|elem| match elem {
                Element::Way(way) => way.polygon().map(|poly| {
                    (
                        Self {
                            tags: way.tags,
                            geometry: poly.into(),
                        },
                        WorldPosition(way.bounds.centroid()),
                        DecorateRequest,
                    )
                }),
                Element::Relation(_rel) => {
                    // TODO: implement
                    None
                }
                Element::Node(_) => None,
            })
            .collect())
    }
}
