use bevy::prelude::*;
use bevy_egui::{
    egui::{vec2, Area},
    EguiContexts,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            WorldInspectorPlugin::new().run_if(|state: Res<State>| state.show_inspector),
        )
        .init_resource::<State>()
        .add_systems(Update, (toggles,));
    }
}

#[derive(Resource, Default)]
struct State {
    show_inspector: bool,
}

fn toggles(mut state: ResMut<State>, mut egui_contexts: EguiContexts) {
    let ctx = egui_contexts.ctx_mut();

    Area::new("toggles")
        .fixed_pos(ctx.available_rect().left_bottom() - vec2(0., 20.))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.);
                ui.checkbox(&mut state.show_inspector, "Inspector");
            });
        });
}
