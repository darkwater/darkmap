pub mod label;
pub mod tooltip;

use bevy::{prelude::*, ui::UiSystem};
use bevy_egui::{
    egui::{FontData, FontDefinitions, FontFamily},
    EguiContexts,
};

#[derive(Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, tooltip::show_tooltip)
            .add_systems(PreUpdate, label::update_labels.before(UiSystem::Layout));
    }
}

fn setup(mut egui_contexts: EguiContexts) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "noto-sans-jp".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/NotoSansJP-Regular.ttf")),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .push("noto-sans-jp".to_owned());

    egui_contexts.ctx_mut().set_fonts(fonts);
}
