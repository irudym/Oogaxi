use crate::{
    physics::PhysicalTranslation,
    states::{AppState, IsPaused},
};
use bevy::prelude::*;

pub struct ZooPlugin;

impl Plugin for ZooPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_zoo)
            .add_systems(
                Update,
                (
                    panic_button,
                    tick_lifetimes,
                    make_a_rock,
                    apply_velocity,
                    screen_wrap,
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
struct Player;

#[derive(Component)]
struct Passenger;

#[derive(Component)]
struct Platform;

#[derive(Component)]
struct Pterodactyl;

/// Data component: velocity in world units/second
#[derive(Component)]
struct Velocity(Vec2);

/// Capability component: wrap around screen edges when out of bounds
#[derive(Component)]
struct ScreenWrap;

/// Life time component: remove entity after particular time
#[derive(Component)]
struct LifeTime(Timer);

/// Rock component
#[derive(Component)]
struct Rock;

fn setup_zoo(mut commands: Commands, asset_server: Res<AssetServer>) {
    // three platforms: static
    for (x, y) in [(-400.0, -200.0), (0.0, -250.0), (400.0, -150.0)] {
        commands.spawn((
            Platform,
            Sprite::from_image(asset_server.load("sprites/platform.png")),
            Transform::from_xyz(x, y, 1.0),
            DespawnOnExit(AppState::InGame),
        ));
    }

    // a passenger waiting on each platform
    for (x, y) in [(-400.0, -50.0), (0.0, -100.0), (400.0, -20.0)] {
        commands.spawn((
            Passenger,
            Sprite::from_image(asset_server.load("sprites/passenger.png")),
            Transform::from_xyz(x, y, 0.5),
            DespawnOnExit(AppState::InGame),
        ));
    }

    // eight pterodactyls drifting with different velocities
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;

        commands.spawn((
            Pterodactyl,
            ScreenWrap,
            Velocity(Vec2::from_angle(angle) * 120.0),
            Sprite::from_image(asset_server.load("sprites/ptero.png")),
            Transform::from_xyz(0.0, 100.0, 8.0),
            DespawnOnExit(AppState::InGame),
        ));
    }
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

/// SPACE: panic the passengers (give them Velocity + ScreenWrap)
/// R: calm them down (remove those components)
fn panic_button(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    passengers: Query<Entity, With<Passenger>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for (i, entity) in passengers.iter().enumerate() {
            let angle = i as f32 * 2.1;
            commands
                .entity(entity)
                .insert((Velocity(Vec2::from_angle(angle) * 120.0), ScreenWrap));
        }
    }

    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &passengers {
            commands.entity(entity).remove::<(Velocity, ScreenWrap)>();
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
        commands.spawn((
            Rock,
            Transform::from_xyz(100.0, 300.0, 8.0),
            Velocity(Vec2::new(0.0, -200.0)),
            LifeTime(Timer::from_seconds(1.0, TimerMode::Once)),
            Sprite::from_image(asset_server.load("sprites/rock.png")),
            DespawnOnExit(AppState::InGame),
        ));
    }
}
