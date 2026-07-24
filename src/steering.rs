use bevy::prelude::*;

/// Accelerate towards target at up to max_speed, turning no harder than
/// max_force allows. Returns a velocity change for this tick
pub fn seek(pos: Vec2, vel: Vec2, target: Vec2, max_speed: f32, max_force: f32, dt: f32) -> Vec2 {
    let desired = (target - pos).normalize_or_zero() * max_speed;
    (desired - vel).clamp_length_max(max_force) * dt
}

/// Seek that brakes: inside "slow_radius", desired speed scales down to zero at the target.
pub fn arrive(
    pos: Vec2,
    vel: Vec2,
    target: Vec2,
    max_speed: f32,
    max_force: f32,
    slow_radius: f32,
    dt: f32,
) -> Vec2 {
    let offset = target - pos;
    let dist = offset.length();
    let speed = if dist < slow_radius {
        max_speed * dist / slow_radius
    } else {
        max_speed
    };
    let desired = offset.normalize_or_zero() * speed;
    (desired - vel).clamp_length_max(max_force) * dt
}
