pub mod view_distance;

use std::f32::consts::{FRAC_PI_3, FRAC_PI_6, FRAC_PI_8, PI};

use bevy::{
    diagnostic::{Diagnostic, RegisterDiagnostic},
    prelude::*,
};
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use geo::{HaversineBearing, HaversineDistance, Point};

use crate::{color, common::WorldPosition, loading::LoadRequest, COLORS};

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

#[derive(Component)]
pub struct MainCamera;

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
            //     near: -5000.,
            //     ..default()
            // }),
            ..default()
        },
        PanOrbitCamera {
            pan_sensitivity: 0.8555,
            pan_smoothness: 0.4,
            orbit_smoothness: 0.4,
            modifier_orbit_touchpad: Some(KeyCode::SuperLeft),
            button_pan: MouseButton::Left,
            button_orbit: MouseButton::Right,
            focus_y_upper_limit: Some(0.),
            focus_y_lower_limit: Some(0.),
            zoom_upper_limit: Some(900.),
            zoom_lower_limit: Some(10.),
            // zoom_upper_limit: Some(1.),
            // zoom_lower_limit: Some(0.05),
            // alpha_upper_limit: Some(0.),
            // alpha_lower_limit: Some(0.),
            // beta_upper_limit: Some(FRAC_PI_2 - 0.01),
            // beta_lower_limit: Some(FRAC_PI_2 - 0.01),
            ..default()
        },
        // AtmosphereCamera::default(),
        FogSettings {
            color: Color::rgba(0.35, 0.48, 0.66, 1.0),
            directional_light_color: Color::rgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::Linear { start: 800., end: 2500. },
            // falloff: FogFalloff::from_visibility_colors(
            //     1000.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
            //     Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
            //     Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
            // ),
        },
        MainCamera,
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(
            Quat::from_rotation_x(-FRAC_PI_3) * Quat::from_rotation_y(FRAC_PI_8),
        ),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Box::new(4000., 4000., 4000.).into()),
        material: materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::rgba(0.35, 0.48, 0.66, 1.0),
            ..default()
        }),
        transform: Transform::from_scale(Vec3::splat(-1.)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.5, 0.5, 0.5),
        brightness: 0.6,
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 5000., subdivisions: 0 }.into()),
        material: materials.add(StandardMaterial {
            unlit: true,
            reflectance: 0.,
            base_color: color(COLORS.base).with_a(0.5),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
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
