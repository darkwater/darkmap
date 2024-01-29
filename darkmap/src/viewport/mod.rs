pub mod view_distance;

use std::f32::consts::PI;

use bevy::{
    diagnostic::{Diagnostic, RegisterDiagnostic},
    prelude::*,
};
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
            .register_diagnostic(
                Diagnostic::new(
                    view_distance::VIEW_DISTANCE_DIAGNOSTIC,
                    "view distance handling",
                    10,
                )
                .with_suffix(view_distance::VIEW_DISTANCE_DIAGNOSTIC_SUFFIX),
            )
            .add_systems(Startup, setup)
            .add_systems(Update, (give_position, view_distance::update_visibility));
    }
}

#[derive(Resource)]
pub struct OriginCoordinate(pub Point);

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut transform = Transform::from_translation(Vec3::new(0., 500., 200.));
    transform.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection { far: 5000., ..default() }),
            // projection: Projection::Orthographic(OrthographicProjection {
            //     far: 5000.,
            //     scale: 0.0001,
            //     ..default()
            // }),
            ..default()
        },
        PanOrbitCamera {
            pan_smoothness: 0.2,
            button_pan: MouseButton::Left,
            button_orbit: MouseButton::Right,
            focus_y_upper_limit: Some(0.),
            focus_y_lower_limit: Some(0.),
            zoom_upper_limit: Some(900.),
            zoom_lower_limit: Some(10.),
            ..default()
        },
        AtmosphereCamera::default(),
        FogSettings {
            color: Color::rgb(0.53, 0.81, 0.92),
            directional_light_color: Color::rgba(1.0, 0.95, 0.75, 1.),
            directional_light_exponent: 200.0,
            // falloff: FogFalloff::from_visibility_colors(
            //     2_000.,
            //     Color::rgb(0., 0., 0.),
            //     Color::rgb(0.8, 0.844, 1.0),
            // ),
            falloff: FogFalloff::Atmospheric {
                extinction: Vec3::new(0.0001, 0.0001, 0.0001),
                inscattering: Vec3::new(0.0001, 0.0001, 0.0001),
            },
        },
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 4.)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.5, 0.5, 0.5),
        brightness: 0.5,
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 5000., subdivisions: 0 }.into()),
        material: materials.add(Color::rgba(0.2, 0.2, 0.2, 0.5).into()),
        transform: Transform::from_translation(Vec3::new(0., -1., 0.)),
        ..default()
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
