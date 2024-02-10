mod decorate;
mod material;

use anyhow::Context;
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;
use bevy_mod_picking::{focus::PickingInteraction, selection::PickSelection};
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
            .add_systems(Update, (decorate::decorate_building, update_outline));
    }
}

fn update_outline(
    mut query: Query<
        (&mut OutlineVolume, &PickingInteraction, &PickSelection),
        Or<(Changed<PickingInteraction>, Changed<PickSelection>)>,
    >,
) {
    for (mut outline, interaction, selection) in &mut query {
        if let PickingInteraction::Pressed = interaction {
            outline.visible = true;
            outline.colour = Color::rgb(1., 1., 0.);
        } else if selection.is_selected {
            outline.visible = true;
            outline.colour = Color::rgb(1., 1., 1.);
        } else if let PickingInteraction::Hovered = interaction {
            outline.visible = true;
            outline.colour = Color::rgb(1., 0., 0.);
        } else {
            outline.visible = false;
        }
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
                        WorldPosition(way.bounds.unwrap().centroid()),
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
