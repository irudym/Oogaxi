use crate::{
    collision::{Collider, Hazard},
    levels::LevelOwned,
    physics::{PhysicalTranslation, PreviousPhysicalTranslation, Velocity},
    states::{AppState, IsPaused},
    z::z,
};
use bevy::prelude::*;

pub struct ZooPlugin;

impl Plugin for ZooPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_lifetimes,
                make_a_rock,
                /* apply_velocity, */ screen_wrap,
            )
                .chain()
                .run_if(in_state(IsPaused::Running)),
        );
        app.add_observer(|add: On<Add, Passenger>| {
            info!("A passenger appeared: {}", add.entity);
        });

        app.add_observer(|remove: On<Remove, Passenger>| {
            info!("A passenger left us: {}", remove.entity)
        });
    }
}

#[derive(Component)]
struct Passenger;

#[derive(Component, Default)]
pub struct Platform;

#[derive(Component)]
struct Pterodactyl;

/// Capability component: wrap around screen edges when out of bounds
#[derive(Component)]
struct ScreenWrap;

/// Life time component: remove entity after particular time
#[derive(Component)]
struct LifeTime(Timer);

/// Rock component
#[derive(Component)]
struct Rock;

pub fn spawn_pterodactyl(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pos: Vec2,
    vel: Vec2,
) {
    commands.spawn((
        Pterodactyl,
        Velocity(vel),
        Sprite::from_image(asset_server.load("sprites/ptero.png")),
        Transform::from_translation(pos.extend(z::HAZARD)),
        Hazard { radius: 16.0 },
        PhysicalTranslation(pos),
        PreviousPhysicalTranslation(pos),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}

pub fn spawn_platform(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pos: Vec2,
    half: Vec2,
) {
    commands.spawn((
        Platform,
        Sprite::from_image(asset_server.load("sprites/platform.png")),
        Transform::from_translation(pos.extend(z::PLATFORM)),
        Collider { half },
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}

// ********
// Systems
// ********

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity), Without<PhysicalTranslation>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}

fn screen_wrap(mut query: Query<&mut Transform, With<ScreenWrap>>, window: Single<&Window>) {
    let half = Vec2::new(window.width(), window.height()) / 2.0;
    for mut transform in &mut query {
        let p = &mut transform.translation;
        if p.x > half.x {
            p.x = -half.x
        }
        if p.x < -half.x {
            p.x = half.x
        }
        if p.y > half.y {
            p.y = -half.y
        }
        if p.y < -half.y {
            p.y = half.y
        }
    }
}

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

fn make_a_rock(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    if keys.just_pressed(KeyCode::KeyY) {
        let start = Vec3::new(100.0, 100.0, z::HAZARD);
        commands.spawn((
            Rock,
            Transform::from_translation(start),
            Velocity(Vec2::new(0.0, -200.0)),
            LifeTime(Timer::from_seconds(1.0, TimerMode::Once)),
            Sprite::from_image(asset_server.load("sprites/rock.png")),
            Hazard { radius: 12.0 },
            PhysicalTranslation(start.truncate()),
            PreviousPhysicalTranslation(start.truncate()),
            DespawnOnExit(AppState::InGame),
        ));
    }
}
