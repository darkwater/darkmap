use bevy::{
    core::Name,
    ecs::system::{Query, Res},
    hierarchy::Children,
};
use bevy_egui::{
    egui::{show_tooltip_at_pointer, Id},
    EguiContexts,
};
use bevy_mod_picking::focus::HoverMap;

pub fn show_tooltip(
    names: Query<&Name>,
    children: Query<&Children>,
    hovers: Res<HoverMap>,
    mut egui_contexts: EguiContexts,
) {
    let ctx = egui_contexts.ctx_mut();

    let mut tooltip_counter = 0;
    let mut tooltip = move || {
        tooltip_counter += 1;
        Id::new("tooltip").with(tooltip_counter)
    };

    for hovers in hovers.values() {
        for entity in hovers.keys() {
            show_tooltip_at_pointer(ctx, tooltip(), |ui| {
                if let Ok(name) = names.get(*entity) {
                    ui.label(name.to_string());
                }

                if let Ok(children) = children.get(*entity) {
                    for name in children.iter().filter_map(|entity| names.get(*entity).ok()) {
                        ui.label(name.to_string());
                    }
                }
            });
        }
    }
}
