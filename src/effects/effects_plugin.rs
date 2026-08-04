use crate::camera::camera_follow;
use crate::effects::effects::*;
use crate::states::IsPaused;
use bevy::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Trauma>().add_systems(
            Update,
            (
                add_trauma_on_events,
                apply_shake
                    .after(camera_follow)
                    .run_if(in_state(IsPaused::Running)),
            ),
        );
    }
}
