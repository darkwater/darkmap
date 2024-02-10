#![feature(array_windows)]
#![feature(associated_type_bounds)]
#![feature(impl_trait_in_assoc_type)]
#![feature(iter_array_chunks)]
#![feature(iter_map_windows)]
#![allow(clippy::type_complexity)]

mod buildings;
mod common;
mod debug;
mod loading;
mod overpass;
mod poi;
mod roads;
mod ui;
mod viewport;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_egui::EguiPlugin;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use catppuccin::{Flavour, FlavourColours};

use self::{
    buildings::BuildingsPlugin, debug::DebugPlugin, poi::PoiPlugin, roads::RoadsPlugin,
    ui::UiPlugin, viewport::ViewportPlugin,
};

const COLORS: FlavourColours = Flavour::Frappe.colours();

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
            OutlinePlugin,
            UiPlugin,
            DebugPlugin,
        ))
        .add_plugins((FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin))
        .add_plugins((BuildingsPlugin, PoiPlugin, RoadsPlugin, ViewportPlugin))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
