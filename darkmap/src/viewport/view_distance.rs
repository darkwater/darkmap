use bevy::prelude::*;

#[derive(Component)]
pub struct ViewDistance(pub f32);

pub(super) fn update_visibility(
    mut query: Query<(&mut Visibility, &GlobalTransform, &ViewDistance)>,
    camera: Query<&Transform, With<Camera>>,
) {
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
}
