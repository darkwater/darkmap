use bevy::prelude::*;

use crate::viewport::MainCamera;

#[derive(Component)]
pub struct Label {
    pub follow: Entity,
}

pub fn update_labels(
    mut labels: Query<(&Label, &mut Style)>,
    targets: Query<&GlobalTransform>,
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
) {
    let Ok((camera_transform, camera)) = camera_query.get_single() else {
        return;
    };

    for (label, mut label_transform) in &mut labels {
        let Ok(target_transform) = targets.get(label.follow) else {
            continue;
        };

        let Some(pos) = camera.world_to_viewport(camera_transform, target_transform.translation())
        else {
            continue;
        };

        label_transform.left = Val::Px(pos.x);
        label_transform.top = Val::Px(pos.y);
    }
}
