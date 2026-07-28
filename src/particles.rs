use crate::{
    messages::{CopterDamaged, Landed},
    physics::PhysicalTranslation,
    player::Player,
    states::{AppState, IsPaused},
};
use bevy::prelude::*;
use rand::RngExt;

use crate::{colors::GameColors, game_rand::GameRng, physics::Velocity, z::z};

const MAX_PARTICLES: usize = 200;
const DUST_GRAVITY: f32 = 60.0;
const DUST_DRAG: f32 = 3.5;

#[derive(Component)]
struct Particle;

#[derive(Component)]
struct Fade(f32);

#[derive(Component)]
struct LifeTime(Timer);

fn spawn_dust(
    commands: &mut Commands,
    // assets: &GameAssets, - in case of using sprite from image
    rng: &mut GameRng,
    at: Vec2,
    intensity: f32,
) {
    let pos = Vec3::new(at.x, at.y - 16.0, z::FX);
    let count = (8.0 + intensity * 40.0) as usize;
    for _ in 0..count {
        let side = if rng.0.random_bool(0.5) { 1.0 } else { -1.0 };
        let angle_from_horizontal: f32 = rng.0.random_range(0.1..0.8); // 6 - 46 grad
        let dir = Vec2::new(
            side * angle_from_horizontal.cos(),
            angle_from_horizontal.sin(),
        );
        let speed = rng.0.random_range(60.0..140.0) * (0.4 + intensity * 0.6);
        let vel = dir * speed;

        commands.spawn((
            Particle,
            Fade(rng.0.random_range(0.5..1.0)),
            Sprite::from_color(GameColors::DUST, Vec2::splat(2.0)),
            Transform::from_translation(pos),
            Velocity(vel),
            LifeTime(Timer::from_seconds(
                rng.0.random_range(0.3..0.7),
                TimerMode::Once,
            )),
            DespawnOnExit(AppState::InGame),
        ));
    }
}

/// Lifetime counter
fn tick_lifetimes(
    mut commands: Commands,
    mut query: Query<(&mut LifeTime, Entity)>,
    time: Res<Time>,
) {
    for (mut lifetime, entity) in &mut query {
        lifetime.0.tick(time.delta());

        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Gravity and fade.
fn update_particles(
    time: Res<Time>,
    mut particles: Query<
        (&mut Transform, &mut Velocity, &mut Sprite, &LifeTime, &Fade),
        With<Particle>,
    >,
) {
    let dt = time.delta_secs();
    for (mut tf, mut vel, mut sprite, life, fade) in &mut particles {
        vel.0.y -= DUST_GRAVITY * dt;
        vel.0 *= (-DUST_DRAG * dt).exp();
        tf.translation += vel.0.extend(0.0) * dt;
        tf.scale = Vec3::splat(1.0 + life.0.fraction() * 0.5);
        sprite.color = sprite.color.with_alpha(fade.0 * (1.0 - life.0.fraction())); //fade out
    }
}

/// Limit amount of the particles on the screen
fn enforce_particle_cap(
    mut commands: Commands,
    particles: Query<(Entity, &LifeTime), With<Particle>>,
) {
    let count = particles.iter().count();
    if count <= MAX_PARTICLES {
        return;
    }
    let mut by_age: Vec<(Entity, f32)> = particles
        .iter()
        .map(|(e, lifetime)| (e, lifetime.0.fraction()))
        .collect();
    by_age.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (entity, _) in by_age.iter().take(count - MAX_PARTICLES) {
        commands.entity(*entity).despawn();
    }
}

///Spawn particles on impact
fn dust_on_crash(
    mut damage: MessageReader<CopterDamaged>,
    player: Query<&PhysicalTranslation, With<Player>>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    let Ok(pos) = player.single() else {
        return;
    };
    for d in damage.read() {
        let intensity = (d.severity / 400.0).clamp(0.0, 1.5);
        spawn_dust(&mut commands, &mut rng, pos.0, intensity);
    }
}

/// Spawn particles on landing
fn dust_on_landing(
    mut landings: MessageReader<Landed>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    for landing in landings.read() {
        spawn_dust(&mut commands, &mut rng, landing.at, 0.25);
    }
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (tick_lifetimes, update_particles, enforce_particle_cap)
                    .chain()
                    .run_if(in_state(IsPaused::Running)),
                (dust_on_crash, dust_on_landing).run_if(in_state(AppState::InGame)),
            ),
        );
    }
}
