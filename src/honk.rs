use crate::messages::Honked;
use crate::physics::{PhysicalTranslation, SimSet};
use crate::states::IsPaused;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

use crate::input::Action;

/// How long an early Honk press stays valid
const HONK_BUFFER: f32 = 0.12;

#[derive(Component, Default)]
pub struct HonkBuffer(pub Timer);

pub struct HonkPlugin;

impl Plugin for HonkPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Honked>()
            .add_systems(Update, buffer_honk.run_if(in_state(IsPaused::Running)))
            .add_systems(
                FixedUpdate,
                consume_honk.in_set(SimSet::Contact),
                //.before(super::passengers::announce_on_honk),
            );
    }
}

/// Render clock: catch the press, arm the buffer
fn buffer_honk(time: Res<Time>, mut players: Query<(&ActionState<Action>, &mut HonkBuffer)>) {
    for (actions, mut buffer) in &mut players {
        buffer.0.tick(time.delta());
        if actions.just_pressed(&Action::Honk) {
            buffer.0 = Timer::from_seconds(HONK_BUFFER, TimerMode::Once);
        }
    }
}

/// Fixed clock
fn consume_honk(
    mut players: Query<(&PhysicalTranslation, &mut HonkBuffer)>,
    mut honks: MessageWriter<Honked>,
) {
    for (pos, mut buffer) in &mut players {
        if !buffer.0.is_finished() {
            buffer.0 = Timer::default(); // consumed
            honks.write(Honked { at: pos.0 });
        }
    }
}
