mod decorate;
mod material;

use anyhow::Context;
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_mod_picking::{focus::PickingInteraction, selection::PickSelection};
use geo::{Centroid, MultiPolygon};
use serde_json::json;

use crate::{
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
    overpass::Element,
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
        (Entity, Option<&mut OutlineVolume>, &PickingInteraction, &PickSelection),
        (With<Building>, Or<(Changed<PickingInteraction>, Changed<PickSelection>)>),
    >,
    mut commands: Commands,
) {
    for (entity, outline, interaction, selection) in &mut query {
        let mut color = None;

        if let PickingInteraction::Pressed = interaction {
            color = Some(Color::rgb(1., 1., 0.));
        } else if selection.is_selected {
            color = Some(Color::rgb(1., 1., 1.));
        } else if let PickingInteraction::Hovered = interaction {
            color = Some(Color::rgb(1., 0., 0.));
        }

        if let Some(color) = color {
            if let Some(mut outline) = outline {
                outline.visible = true;
                outline.colour = color;
            } else {
                commands.entity(entity).insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 2.,
                        colour: color,
                    },
                    ..default()
                });
            }
        } else if outline.is_some() {
            commands.entity(entity).remove::<OutlineBundle>();
        }
    }
}

#[derive(Component)]
pub struct Building {
    pub geometry: MultiPolygon,
}

impl LoadType for Building {
    type Bundle = impl Bundle;

    async fn load(req: LoadRequest) -> anyhow::Result<Vec<Self::Bundle>> {
        let template = include_str!("../../assets/queries/buildings.ovp");

        let query = handlebars::Handlebars::new()
            .render_template(template, &json!({ "bbox": req.bbox() }))
            .context("Failed to render query")?;

        let res = crate::overpass::load(&query)
            .await
            .context("Failed to load buildings")?;

        Ok(res
            .elements
            .into_iter()
            .flat_map(|elem| match elem {
                Element::Way(way) => way.polygon().map(|poly| {
                    (
                        Self { geometry: poly.into() },
                        way.tags,
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
