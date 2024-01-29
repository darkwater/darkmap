use std::time::Instant;

use bevy::{
    diagnostic::{DiagnosticId, Diagnostics},
    prelude::*,
};

#[derive(Component)]
pub struct ViewDistance(pub f32);

pub const VIEW_DISTANCE_DIAGNOSTIC: DiagnosticId =
    DiagnosticId::from_u128(0xD09085FA_DE9F_4876_9812_27B9E48A5BBA);

pub const VIEW_DISTANCE_DIAGNOSTIC_SUFFIX: &str = "us";

pub(super) fn update_visibility(
    mut query: Query<(&mut Visibility, &GlobalTransform, &ViewDistance)>,
    camera: Query<&Transform, With<Camera>>,
    mut diagnostics: Diagnostics,
) {
    let start = Instant::now();

    let camera = camera.get_single().unwrap();

    for (mut visible, transform, ViewDistance(view_dist)) in query.iter_mut() {
        let distance = transform.translation().distance_squared(camera.translation);
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
