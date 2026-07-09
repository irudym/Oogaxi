use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

use crate::{
    input::Action,
    player::{self, PlayerId},
};

/// How long an early Honk press stays valid
const HONK_BUFFER: f32 = 0.12;

#[derive(Component, Default)]
pub struct HonkBuffer(pub Timer);

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
fn consume_honk(mut players: Query<(&PlayerId, &mut HonkBuffer)>) {
    for (id, mut buffer) in &mut players {
        if !buffer.0.finished() {
            buffer.0 = Timer::default();
            info!("HONK from player: {}", id.0);
        }
    }
}
