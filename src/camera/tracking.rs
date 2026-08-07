use bevy::prelude::*;

use crate::camera::GameCamera;

#[derive(Component)]
pub struct TracksGameCamera;

/// One system to track all camera in sync with GameCamera
pub fn sync_tracking_cameras(
    game: Query<&Transform, (With<GameCamera>, Without<TracksGameCamera>)>,
    mut followers: Query<&mut Transform, With<TracksGameCamera>>,
) {
    let Ok(game_tf) = game.single() else {
        return;
    };
    for mut tf in &mut followers {
        *tf = *game_tf;
    }
}
