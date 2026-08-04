use crate::camera::spawn_camera;
use crate::lights::LightMap;
use crate::overlay::overlay::spawn_post_process;
use bevy::prelude::*;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_post_process
                .run_if(resource_exists::<LightMap>)
                .after(spawn_camera),
        );
    }
}
