use std::time::Instant;

use bevy::{
    diagnostic::{DiagnosticId, Diagnostics},
    prelude::*,
};
use bevy_panorbit_camera::PanOrbitCamera;

#[derive(Component)]
pub struct ViewDistance(pub f32);

pub const VIEW_DISTANCE_DIAGNOSTIC: DiagnosticId =
    DiagnosticId::from_u128(0xD09085FA_DE9F_4876_9812_27B9E48A5BBA);

pub const VIEW_DISTANCE_DIAGNOSTIC_SUFFIX: &str = "us";

pub(super) fn update_visibility(
    mut query: Query<(&mut Visibility, &GlobalTransform, &ViewDistance)>,
    camera: Query<(&Transform, &PanOrbitCamera), With<Camera>>,
    mut diagnostics: Diagnostics,
) {
    let start = Instant::now();

    let Ok(camera) = camera.get_single() else {
        return;
    };

    for (mut visible, transform, ViewDistance(view_dist)) in query.iter_mut() {
        let center = camera.1.focus;
        let distance = transform.translation().distance_squared(center);
        let desired = if distance < view_dist.powi(2) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if *visible != desired {
            *visible = desired;
        }
    }

    diagnostics.add_measurement(VIEW_DISTANCE_DIAGNOSTIC, || {
        start.elapsed().as_secs_f64() * 1024. * 1024.
    });
}
