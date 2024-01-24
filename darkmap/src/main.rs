use bevy::prelude::*;
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AtmospherePlugin,
            DefaultPickingPlugins,
            EguiPlugin,
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle::default(),
        PanOrbitCamera {
            button_pan: MouseButton::Middle,
            button_orbit: MouseButton::Right,
            ..default()
        },
        AtmosphereCamera::default(),
    ));
}
