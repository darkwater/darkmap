use std::time::Instant;

use anyhow::Context;
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineMeshExt, OutlineVolume};
use bevy_mod_picking::prelude::*;
use geo::{Contains, HaversineDistance};
use overpass::{Element, Tags};
use serde_json::json;

use crate::{
    buildings::Building,
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
    viewport::view_distance::ViewDistance,
};

#[derive(Default)]
pub struct PoiPlugin;

impl Plugin for PoiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LoadingPlugin::<PointOfInterest>::new())
            .add_systems(Update, (decorate_poi, move_up));
    }
}

#[derive(Component)]
pub struct PointOfInterest {
    pub tags: Tags,
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
    let start = Instant::now();
    for (entity, _poi) in &mut query.iter() {
        if start.elapsed().as_millis() > 2 {
            break;
        }

        let mut mesh = Mesh::from(shape::UVSphere { radius: 1., sectors: 4, stacks: 2 });
        let _res = mesh.generate_outline_normals();

        commands.entity(entity).remove::<DecorateRequest>().insert((
            meshes.add(mesh),
            materials.add(StandardMaterial {
                base_color: Color::rgb(1., 0., 0.),
                unlit: true,
                ..Default::default()
            }),
            ViewDistance(1000.),
            On::<Pointer<Over>>::listener_commands_mut(|_, cmds| {
                cmds.insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 2.,
                        colour: Color::rgb(1., 1., 1.),
                    },
                    ..default()
                });
            }),
            On::<Pointer<Out>>::listener_commands_mut(|_, cmds| {
                cmds.remove::<OutlineBundle>();
            }),
        ));
    }
}

fn move_up(
    mut pois: Query<(&WorldPosition, &mut Transform), With<PointOfInterest>>,
    buildings: Query<
        (&WorldPosition, &Building),
        (With<DecorateRequest>, Without<PointOfInterest>),
    >,
) {
    // for (poi_pos, mut transform) in &mut pois {
    //     for (building_pos, building) in &buildings {
    //         if building_pos.0.haversine_distance(&poi_pos.0) > 100. {
    //             continue;
    //         }

    //         if building.geometry.contains(&poi_pos.0) {
    //             transform.translation.y = building.tags.building_height().unwrap_or(10.);
    //             break;
    //         }
    //     }
    // }
}
