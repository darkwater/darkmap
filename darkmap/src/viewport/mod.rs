pub mod view_distance;

use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_real_timer};
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use geo::{HaversineBearing, HaversineDistance, Point};

use crate::{common::WorldPosition, loading::LoadRequest};

#[derive(Default)]
pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AtmospherePlugin, PanOrbitCameraPlugin))
            .insert_resource(OriginCoordinate(Point::new(139.77137176176117, 35.69967697464613)))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    give_position,
                    view_distance::update_visibility
                        .run_if(on_real_timer(Duration::from_millis(100))),
                ),
            );
    }
}

#[derive(Resource)]
pub struct OriginCoordinate(pub Point);

fn setup(mut commands: Commands) {
    let mut transform = Transform::from_translation(Vec3::new(0., 500., 200.));
    transform.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection { far: 5000., ..default() }),
            ..default()
        },
        PanOrbitCamera {
            button_pan: MouseButton::Left,
            button_orbit: MouseButton::Right,
            ..default()
        },
        AtmosphereCamera::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 4.)),
        ..Default::default()
    });

    commands.spawn(LoadRequest::new(Point::new(139.77137176176117, 35.69967697464613), 1000.));
}

fn give_position(
    query: Query<(Entity, &WorldPosition), Without<Transform>>,
    origin: Res<OriginCoordinate>,
    mut commands: Commands,
) {
    for (entity, WorldPosition(point)) in query.iter() {
        let distance = origin.0.haversine_distance(point);
        let bearing = origin.0.haversine_bearing(*point);

        let ang = bearing.to_radians();

        let translation =
            Vec3::new((distance * ang.sin()) as f32, 0., (distance * -ang.cos()) as f32);

        commands.entity(entity).insert(SpatialBundle {
            transform: Transform::from_translation(translation),
            ..default()
        });
    }
}
