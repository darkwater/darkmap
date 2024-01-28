use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

#[derive(Component)]
pub struct ViewDistance(pub f32);

pub(super) fn update_visibility(
    mut query: Query<(&mut Visibility, &GlobalTransform, &ViewDistance)>,
    camera: Query<&PanOrbitCamera, With<Camera>>,
) {
    let start = std::time::Instant::now();

    let camera = camera.get_single().unwrap();

    for (mut visible, transform, ViewDistance(view_dist)) in query.iter_mut() {
        let distance = transform.translation().distance_squared(camera.focus);
        let desired = if distance < view_dist.powi(2) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if *visible != desired {
            *visible = desired;
        }
    }

    dbg!(start.elapsed());
}
