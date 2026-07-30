use bevy::prelude::*;
use rand::RngExt;

use crate::assets::Sheet;
use crate::bubble::{Bubble, pop_bubble, spawn_bubble};
use crate::collision::Grounded;
use crate::game_rand::GameRng;
use crate::levels::{LevelOwned, Stop, TaxiRegistry};
use crate::messages::PassengerDelivered;
use crate::physics::SimSet;
use crate::player::Player;
use crate::states::AppState;
use crate::z::z;
use crate::{assets::GameAssets, physics::FlightConfig};

pub const HAIL_RADIUS: f32 = 64.0; //4 tiles
const ARRIVE_EPS: f32 = 1.5;
const GLYPH_GRR: usize = 10;

#[derive(Component)]
pub struct Passenger {
    pub origin: u8,
    pub destination: u8,
}

/// The states
#[derive(Component)]
pub struct Emerging(pub Timer);

#[derive(Component)]
pub struct WalkingToSign;

#[derive(Component)]
pub struct Waiting;

#[derive(Component)]
pub struct Announcing(pub Timer);

#[derive(Component)]
pub struct Boarding;

#[derive(Component)]
pub struct Riding;

#[derive(Component)]
pub struct Unboarding;

#[derive(Component)]
pub struct Leaving;

// Walk horizontally towards x
#[derive(Component)]
pub struct WalkTo(pub f32);

#[derive(Component)]
pub struct Fare(pub f32);

#[derive(Component)]
pub struct DroppedAt(pub u8);

/// The only legal way between marker-states. 'Out: bundle' - tuples work,
/// so a transition can shed several components atomically.
pub fn transition<Out: Bundle>(commands: &mut Commands, entity: Entity, into: impl Bundle) {
    commands.entity(entity).remove::<Out>().insert(into);
}

fn spawn_passenger(
    commands: &mut Commands,
    assets: &GameAssets,
    stop: &Stop,
    destination: u8,
    sheet: Sheet,
) {
    warn!(
        "Spawn a passenger at address: {}, pos: {}, need to get to: {}",
        stop.address, stop.cave_pos, destination
    );
    commands.spawn((
        Passenger {
            origin: stop.address,
            destination,
        },
        Emerging(Timer::from_seconds(0.4, TimerMode::Once)),
        Sprite {
            image: assets.get(sheet).image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.get(sheet).layout.clone(),
                index: 0,
            }),
            ..default()
        },
        assets.get(sheet).clip("idle"),
        crate::animations::AnimState::default(),
        Transform::from_translation(stop.cave_pos.extend(z::PASSENGER)),
        LevelOwned,
        Visibility::Visible,
        DespawnOnExit(AppState::InGame),
    ));
}

/// Spawns passenger from cave portal on a random cadence, respecting caps
fn passenger_spawner(
    time: Res<Time>,
    cfg: Res<FlightConfig>,
    registry: Res<TaxiRegistry>,
    mut rng: ResMut<GameRng>,
    mut next_spawn: Local<f32>,
    living: Query<&Passenger>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    *next_spawn -= time.delta_secs();
    if *next_spawn > 0.0
        || living.iter().count() > cfg.max_passengers as usize
        || registry.0.len() < 2
    {
        return;
    }
    *next_spawn = rng.0.random_range(cfg.spawn_secs_min..cfg.spawn_secs_max);

    //a stop with nobody already on the ground near it.
    let occupied: Vec<u8> = living.iter().map(|p| p.origin).collect();
    let free: Vec<&Stop> = registry
        .0
        .iter()
        .filter(|s| !occupied.contains(&s.address))
        .collect();
    let Some(origin) = free.get(rng.0.random_range(0..free.len().max(1))).copied() else {
        return;
    };
    let destination = loop {
        let s = &registry.0[rng.0.random_range(0..registry.0.len())];
        if s.address != origin.address {
            break s.address;
        }
    };
    let sheet = match rng.0.random_bool(0.5) {
        true => Sheet::Passenger,
        false => Sheet::Passenger2,
    };
    spawn_passenger(&mut commands, &assets, origin, destination, sheet);
}

// Locomotion for everyone who walks: one system, five states served
fn walk(
    cfg: Res<FlightConfig>,
    time: Res<Time>,
    mut walkers: Query<(&mut Transform, &WalkTo, &mut Sprite)>,
) {
    for (mut tf, target, mut sprite) in &mut walkers {
        let dx = target.0 - tf.translation.x;
        let step = cfg.walk_speed * time.delta_secs();
        tf.translation.x += dx.clamp(-step, step);
        if dx.abs() > 1.0 {
            sprite.flip_x = dx < 0.0;
        }
    }
}

fn arrived(tf: &Transform, target: &WalkTo) -> bool {
    (tf.translation.x - target.0).abs() < ARRIVE_EPS
}

/// Emerging -> WalingToSign: the commute begin, the walk target is the sign captured coordinate
/// the registry answers "how far it is?"
fn finish_emerging(
    time: Res<Time>,
    registry: Res<TaxiRegistry>,
    mut emerging: Query<(Entity, &mut Emerging, &Passenger)>,
    mut commands: Commands,
) {
    for (entity, mut e, passenger) in &mut emerging {
        if e.0.tick(time.delta()).is_finished() {
            continue;
        }
        let Some(stop) = registry.by_address(passenger.origin) else {
            continue;
        };

        transition::<Emerging>(
            &mut commands,
            entity,
            (WalkingToSign, WalkTo(stop.sign_pos.x)),
        );
    }
}

/// WalkingToSign -> Waiting
fn arrive_at_sign(
    walkers: Query<(Entity, &Transform, &WalkTo), With<WalkingToSign>>,
    mut commands: Commands,
) {
    for (entity, tf, target) in &walkers {
        if arrived(tf, target) {
            transition::<(WalkingToSign, WalkTo)>(&mut commands, entity, Waiting);
        }
    }
}

/// Waiting -> Announcing: as soon as the passenger reaches the sign, raise the destination bubble
/// TODO: announcing should have some min time (2 sec) before the passenger start boarding.
pub fn announce_at_sign(
    waiting: Query<(Entity, &Passenger), With<Waiting>>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    for (entity, passenger) in &waiting {
        let bubble = spawn_bubble(
            &mut commands,
            &assets,
            entity,
            passenger.destination as usize,
        );
        transition::<Waiting>(
            &mut commands,
            entity,
            (
                Announcing(Timer::from_seconds(2.0, TimerMode::Once)),
                Bubble(bubble),
            ),
        );
    }
}

/// (Waiting | Announcing) -> Boarding: the copter landed at their stop
/// Decision 2 lives in this Or<> - announcement is information, not permission
fn board_on_landing(
    registry: Res<TaxiRegistry>,
    mut candidates: Query<
        (Entity, &Passenger, Option<&Bubble>, Option<&mut Announcing>),
        Or<(With<Waiting>, With<Announcing>)>,
    >,
    aboard: Query<(), (With<Passenger>, Or<(With<Boarding>, With<Riding>)>)>,
    player: Query<(&Transform, Has<Grounded>), With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Ok(copter) = player.single() else {
        return;
    };

    if !copter.1 {
        return;
    }

    if !aboard.is_empty() {
        return;
    }

    let Some(stop) = registry.stop_near(copter.0.translation.truncate(), HAIL_RADIUS) else {
        return;
    };

    if let Some((entity, _, bubble, mut announcing)) = candidates
        .iter_mut()
        .find(|(_, p, _, _)| p.origin == stop.address)
    {
        if let Some(ann) = &mut announcing {
            // tick timer only then the copter landed
            if !ann.0.tick(time.delta()).is_finished() {
                return;
            }
        }
        pop_bubble(&mut commands, entity, bubble);
        transition::<(Waiting, Announcing)>(
            &mut commands,
            entity,
            (Boarding, WalkTo(copter.0.translation.x)),
        );
    }
}

/// Boarding -> Riding - or aborted back to Waiting if te copter left
fn finish_boarding(
    cfg: Res<FlightConfig>,
    registry: Res<TaxiRegistry>,
    boarding: Query<(Entity, &Transform, &WalkTo, &Passenger), With<Boarding>>,
    player: Query<(&Transform, Has<Grounded>), With<Player>>,
    mut commands: Commands,
) {
    let Ok((copter, grounded)) = player.single() else {
        return;
    };
    for (entity, tf, target, passenger) in &boarding {
        // Copter took off mid-walk? Shrug, walk back.
        if !grounded || (copter.translation.x - target.0).abs() > 12.0 {
            let sign_x = registry.by_address(passenger.origin).map(|s| s.sign_pos.x);
            transition::<(Boarding, WalkTo)>(
                &mut commands,
                entity,
                (WalkingToSign, WalkTo(sign_x.unwrap_or(tf.translation.x))),
            );
            continue;
        }
        if arrived(tf, target) {
            let fare = registry.fare_between(passenger.origin, passenger.destination, &cfg);
            transition::<(Boarding, WalkTo)>(
                &mut commands,
                entity,
                (Riding, Fare(fare), Visibility::Hidden),
            );
        }
    }
}

/// The economy's heartbeat, fixed clock, so pause freezes the meter free.
fn fare_decay(cfg: Res<FlightConfig>, time: Res<Time>, fares: Query<&mut Fare, With<Riding>>) {
    for mut fare in fares {
        fare.0 = (fare.0 - cfg.fare_decay * time.delta_secs()).max(cfg.fare_min);
    }
}

/// Riding -> Unboarding: any registered stop unboarding
fn unboard_on_landing(
    //mut landings: MessageReader<Landed>,
    registry: Res<TaxiRegistry>,
    riding: Query<(Entity, &Passenger), With<Riding>>,
    player: Query<(&Transform, Has<Grounded>), With<Player>>,
    mut commands: Commands,
) {
    // Passengers need to unboard the copter event when it's crashed.
    let Ok((copter, grounded)) = player.single() else {
        return;
    };

    let Some(stop) = registry.stop_near(copter.translation.truncate(), HAIL_RADIUS) else {
        return;
    };

    //check if the copter grounded
    if !grounded {
        return;
    }
    for (entity, passenger) in &riding {
        let mut drop =
            Transform::from_translation(copter.translation.truncate().extend(z::PASSENGER));
        drop.translation.y = stop.sign_pos.y; //feet on the pad, not mid rotor
        // check that landing address is different than origin, otherwise keep the passenger in the copter.
        if stop.address == passenger.origin {
            continue;
        }

        transition::<Riding>(
            &mut commands,
            entity,
            (
                Unboarding,
                DroppedAt(stop.address),
                Visibility::Visible,
                drop,
                WalkTo(stop.sign_pos.x),
            ),
        );
    }
}

/// Unboarding -> Leaving - the judgment. Right address pays, wrong address say something rude and pay nothing.
fn finish_unboarding(
    registry: Res<TaxiRegistry>,
    assets: Res<GameAssets>,
    done: Query<(Entity, &Transform, &WalkTo, &Passenger, &Fare, &DroppedAt), With<Unboarding>>,
    mut delivered: MessageWriter<PassengerDelivered>,
    mut commands: Commands,
) {
    for (entity, tf, target, passenger, fare, dropped) in &done {
        if !arrived(tf, target) {
            continue;
        }
        if dropped.0 == passenger.destination {
            delivered.write(PassengerDelivered {
                fare: fare.0.round() as u32,
            });
        } else {
            let bubble = spawn_bubble(&mut commands, &assets, entity, GLYPH_GRR);
            commands.entity(entity).insert(Bubble(bubble));
        }
        // walk of shame / walk of pay: to the DROP-OFF stop's cave; the designer may prefer the long walk).
        let cave_x = registry.by_address(dropped.0).map(|s| s.cave_pos.x);
        transition::<(Unboarding, WalkTo, Fare, DroppedAt)>(
            &mut commands,
            entity,
            (Leaving, WalkTo(cave_x.unwrap_or(tf.translation.x))),
        );
    }
}

/// Leaving -> gone
fn vanish_into_cave(
    done: Query<(Entity, &Transform, &WalkTo), With<Leaving>>,
    mut commands: Commands,
) {
    for (entity, tf, target) in done {
        if arrived(tf, target) {
            commands.entity(entity).despawn();
        }
    }
}

// World objects

pub fn spawn_sign(commands: &mut Commands, assets: &GameAssets, pos: Vec2, address: u8) {
    commands.spawn((
        Sprite {
            image: assets.get(crate::assets::Sheet::Signs).image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.get(crate::assets::Sheet::Signs).layout.clone(),
                index: (address as usize).saturating_sub(1),
            }),
            ..default()
        },
        Transform::from_translation(pos.extend(z::PASSENGER - 1.0)),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}

pub fn spawn_cave(commands: &mut Commands, assets: &GameAssets, pos: Vec2) {
    commands.spawn((
        Transform::from_translation(pos.extend(z::PASSENGER - 1.0)),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}

pub struct PassengerPlugin;

impl Plugin for PassengerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                passenger_spawner,
                walk,
                finish_emerging,
                arrive_at_sign,
                // announce_on_honk,
                announce_at_sign,
                board_on_landing,
                finish_boarding,
                fare_decay,
                unboard_on_landing,
                finish_unboarding,
                vanish_into_cave,
            )
                .chain()
                .in_set(SimSet::Contact),
        );
        #[cfg(feature = "dev")]
        app.add_systems(Update, draw_debug_layer);
    }
}

fn draw_debug_layer(mut gizmos: Gizmos, waiting: Query<&Transform, With<Announcing>>) {
    for tf in waiting {
        gizmos.circle_2d(
            Isometry2d::from_translation(tf.translation.truncate()),
            HAIL_RADIUS,
            Color::srgb(0.4, 0.3, 1.0),
        );
    }
}
