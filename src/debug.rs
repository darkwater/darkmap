use bevy::prelude::*;
use bevy_egui::{
    egui::{vec2, Area},
    EguiContexts,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::focus::HoverMap;

#[derive(Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            WorldInspectorPlugin::new().run_if(|state: Res<State>| state.show_inspector),
        )
        .init_resource::<State>()
        .add_systems(
            Update,
            (toggles, show_normals.run_if(|state: Res<State>| state.show_normals)),
        );
    }
}

#[derive(Resource, Default)]
struct State {
    show_inspector: bool,
    show_normals: bool,
}

fn toggles(mut state: ResMut<State>, mut egui_contexts: EguiContexts) {
    let ctx = egui_contexts.ctx_mut();

    Area::new("toggles")
        .fixed_pos(ctx.available_rect().left_bottom() - vec2(0., 20.))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.);
                ui.checkbox(&mut state.show_inspector, "Inspector");
                ui.checkbox(&mut state.show_normals, "Normals");
            });
        });
}

fn show_normals(
    query: Query<(&Transform, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
    hovers: Res<HoverMap>,
    mut gizmos: Gizmos,
) {
    for hovers in hovers.values() {
        for entity in hovers.keys() {
            let Ok((transform, mesh)) = query.get(*entity) else {
                continue;
            };

            let Some(mesh) = meshes.get(mesh) else {
                continue;
            };

            let Some(positions) = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|v| v.as_float3())
            else {
                continue;
            };

            let Some(normals) = mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .and_then(|v| v.as_float3())
            else {
                continue;
            };

            for (&position, &normal) in positions.iter().zip(normals) {
                let position = transform.transform_point(position.into());

                gizmos.line(
                    position,
                    position + Vec3::from(normal).normalize() * 3.,
                    Color::rgb(1., 0., 0.),
                );
            }
        }
    }
}
