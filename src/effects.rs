use bevy::prelude::*;
use rand::RngExt;

use crate::{
    camera::GameCamera,
    game_rand::GameRng,
    messages::{CopterDamaged, Landed},
};

#[derive(Resource, Default)]
pub struct Trauma(pub f32);

pub fn add_trauma_on_events(
    mut damage: MessageReader<CopterDamaged>,
    mut landing: MessageReader<Landed>,
    mut trauma: ResMut<Trauma>,
) {
    for d in damage.read() {
        trauma.0 = (trauma.0 + (d.severity / 400.0).min(0.6)).min(1.0);
    }
    for _ in landing.read() {
        trauma.0 = (trauma.0 * 0.15).min(1.0);
    }
}

/// Apply shake on impact
/// Squirrel Eiserloh model
pub fn apply_shake(
    time: Res<Time>,
    mut trauma: ResMut<Trauma>,
    mut rng: ResMut<GameRng>,
    mut camera: Query<&mut Transform, With<GameCamera>>,
) {
    if trauma.0 <= 0.000001 {
        return; // not just a CPU saving: don't touch the camera when not shaking!
    }
    trauma.0 = (trauma.0 - time.delta_secs() * 1.2).max(0.0);
    let Ok(mut tf) = camera.single_mut() else {
        return;
    };
    let shake = trauma.0 * trauma.0;
    let angle = rng.0.random_range(-1.0..1.0) * shake * 0.05;
    let offset =
        Vec2::new(rng.0.random_range(-1.0..1.0), rng.0.random_range(-1.0..1.0)) * shake * 8.0;
    tf.rotation = Quat::from_rotation_z(angle);
    tf.translation.x += offset.x;
    tf.translation.y += offset.y;
}
