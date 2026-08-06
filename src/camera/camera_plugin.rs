use crate::{camera::camera::*, states::AppState};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraConfig>()
            .init_resource::<CameraConfig>()
            .init_resource::<LevelBounds>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_follow.run_if(in_state(AppState::InGame)),
                    parallax.after(camera_follow),
                ),
            );
    }
}
