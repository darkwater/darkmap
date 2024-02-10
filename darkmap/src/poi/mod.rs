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
    ui::label::Label,
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
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if query.iter().next().is_none() {
        return;
    }

    let font = assets.load("fonts/NotoSansJP-Regular.ttf");

    let start = Instant::now();
    for (entity, poi) in &query {
        if start.elapsed().as_millis() > 1 {
            break;
        }

        let font = font.clone();

        let mut mesh = Mesh::from(shape::UVSphere { radius: 1., sectors: 4, stacks: 2 });
        let _res = mesh.generate_outline_normals();

        let mut cmds = commands.entity(entity);
        cmds.remove::<DecorateRequest>().insert((
            meshes.add(mesh),
            materials.add(StandardMaterial {
                base_color: Color::rgb(1., 0., 0.),
                unlit: true,
                ..Default::default()
            }),
            ViewDistance(1000.),
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
            On::<Pointer<Select>>::commands_mut(move |ev, cmds| {
                let font = font.clone();
                let entity = ev.listener();

                // cmds.entity(entity).add(|mut ent: EntityWorldMut| {
                //     ent.get_mut::<OutlineVolume>().unwrap().colour = Color::rgb(0., 1., 0.);
                // });

                cmds.add(move |world: &mut World| {
                    let font = font.clone();
                    let name = world.get::<Name>(entity).unwrap().as_str();

                    let text = TextBundle::from_section(name, TextStyle {
                        font: font.clone(),
                        font_size: 20.,
                        color: Color::rgb(1., 1., 1.),
                    })
                    .with_text_alignment(TextAlignment::Center);

                    world.spawn((text, Label { follow: entity }));
                });
            }),
            // On::<Pointer<Select>>::listener_component_mut(|_, volume: &mut OutlineVolume| {
            //     volume.colour = Color::rgb(0., 1., 0.);
            // }),
            // On::<Pointer<Select>>::listener_commands_mut(move |_, cmds| {
            //     let font = font.clone();
            //     cmds.add(move |mut ent: EntityWorldMut| {
            //         let Some(name) = ent.get::<Name>() else {
            //             return;
            //         };

            //         let name = name.as_str();

            //         let text = Text::from_section(name, TextStyle {
            //             font: font.clone(),
            //             font_size: 20.,
            //             color: Color::rgb(1., 1., 1.),
            //         });

            //         ent.insert(text);
            //     });
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
            //     cmds.remove::<OutlineBundle>();
            // }),
        ));

        if let Some(name) = poi.tags.name() {
            cmds.insert(Name::new(name.to_string()));
        }
    }
}

fn move_up(
    mut pois: Query<
        (Entity, &WorldPosition, &mut Transform),
        (With<PointOfInterest>, Without<Parent>),
    >,
    buildings: Query<
        (Entity, &WorldPosition, &Transform, &Building),
        (Added<Transform>, Without<PointOfInterest>),
    >,
    mut commands: Commands,
) {
    for (poi_ent, poi_pos, mut poi_transform) in &mut pois {
        for (building_ent, building_pos, building_transform, building) in &buildings {
            // if building_pos.0.haversine_distance(&poi_pos.0) > 100. {
            //     continue;
            // }

            if building.geometry.contains(&poi_pos.0) {
                // poi_transform.translation -= building_transform.translation;
                // poi_transform.translation.y = building.tags.building_height().unwrap_or(10.);

                commands
                    .entity(poi_ent)
                    .remove::<PbrBundle>()
                    .set_parent(building_ent);

                break;
            }
        }
    }
}
