#![feature(array_windows)]
#![feature(associated_type_bounds)]
#![feature(impl_trait_in_assoc_type)]
#![feature(iter_array_chunks)]
#![feature(iter_map_windows)]
#![allow(clippy::type_complexity)]

mod buildings;
mod common;
mod loading;
mod poi;
mod viewport;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::DefaultPickingPlugins;

use self::{buildings::BuildingsPlugin, poi::PoiPlugin, viewport::ViewportPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1920., 1080.),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins,
            EguiPlugin,
            WorldInspectorPlugin::new(),
        ))
        .add_plugins((BuildingsPlugin, PoiPlugin, ViewportPlugin))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
