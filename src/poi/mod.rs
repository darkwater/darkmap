use anyhow::Context;
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineMeshExt, OutlineVolume};
use bevy_mod_picking::prelude::*;
use geo::Contains;
use serde_json::json;

use crate::{
    buildings::Building,
    common::{DecorateRequest, WorldPosition},
    loading::{LoadRequest, LoadType, LoadingPlugin},
    overpass::{Element, Tags},
    ui::label::Label,
    viewport::view_distance::ViewDistance,
    SUBWAY_DEPTH,
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
pub struct PointOfInterest;

impl LoadType for PointOfInterest {
    type Bundle = impl Bundle;

    async fn load(req: LoadRequest) -> anyhow::Result<Vec<Self::Bundle>> {
        let template = include_str!("../../assets/queries/poi.ovp");

        let query = handlebars::Handlebars::new()
            .render_template(template, &json!({ "bbox": req.bbox() }))
            .context("Failed to render query")?;

        let res = crate::overpass::load(&query)
            .await
            .context("Failed to load POI")?;

        Ok(res
            .elements
            .into_iter()
            .flat_map(|elem| {
                if let Element::Node(node) = elem {
                    Some((Self, node.tags, WorldPosition(node.point), DecorateRequest))
                } else {
                    None
                }
            })
            .collect())
    }
}

fn decorate_poi(
    mut query: Query<
        (Entity, &Tags, &mut Transform),
        (With<DecorateRequest>, With<PointOfInterest>),
    >,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if query.iter().next().is_none() {
        return;
    }

    let font = assets.load("fonts/NotoSansJP-Regular.ttf");

    for (entity, tags, mut transform) in query.iter_mut().take(1000) {
        let font = font.clone();

        let mut mesh = Mesh::from(shape::UVSphere { radius: 1., sectors: 4, stacks: 2 });
        let _res = mesh.generate_outline_normals();

        if tags.0.get("subway").is_some() {
            transform.translation.y = SUBWAY_DEPTH;
        }

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
            On::<Pointer<Select>>::commands_mut(move |ev, cmds| {
                let font = font.clone();
                let entity = ev.listener();

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
        ));

        if let Some(name) = tags.name() {
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
        (Entity, &Transform, &Building, &Tags),
        (Added<Transform>, Without<PointOfInterest>),
    >,
    mut commands: Commands,
) {
    for (poi_ent, poi_pos, mut poi_transform) in &mut pois {
        if poi_transform.translation.y != 0. {
            continue;
        }

        for (building_ent, building_transform, building, tags) in &buildings {
            if building.geometry.contains(&poi_pos.0) {
                poi_transform.translation -= building_transform.translation;
                poi_transform.translation.y = tags.building_height().unwrap_or(10.);

                commands
                    .entity(poi_ent)
                    .remove::<Handle<Mesh>>()
                    .set_parent(building_ent);

                break;
            }
        }
    }
}
