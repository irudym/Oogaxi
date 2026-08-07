use crate::lights::lights::{LevelAmbientColor, apply_ambient, flicker_lights, setup_light_map};
use crate::states::IsPaused;
use bevy::prelude::*;

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelAmbientColor::default())
            .add_systems(Startup, setup_light_map)
            .add_systems(
                Update,
                (
                    flicker_lights.run_if(in_state(IsPaused::Running)),
                    apply_ambient.run_if(resource_changed::<LevelAmbientColor>),
                ),
            );
    }
}
