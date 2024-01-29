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
mod roads;
mod viewport;

use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_picking::DefaultPickingPlugins;

use self::{
    buildings::BuildingsPlugin, poi::PoiPlugin, roads::RoadsPlugin, viewport::ViewportPlugin,
};

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
            OutlinePlugin,
        ))
        .add_plugins((
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin,
        ))
        .add_plugins((BuildingsPlugin, PoiPlugin, RoadsPlugin, ViewportPlugin))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
