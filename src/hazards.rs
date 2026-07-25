use crate::assets::GameAssets;
use crate::bubble::{Bubble, BubbleTimer, spawn_bubble};
use crate::collision::clamp_vs_grid;
use crate::{
    animations::AnimState,
    collision::{Collider, Hazard},
    levels::{LevelOwned, TileGrid},
    physics::{PhysicalTranslation, PreviousPhysicalTranslation, SimSet, Velocity},
    player::Player,
    states::{AppState, IsPaused},
    steering,
    z::z,
};
use bevy::prelude::*;

const ARRIVE_EPS: f32 = 6.0;
const TELEGRAPH_SECS: f32 = 0.4;
const RECOVER_SECS: f32 = 1.2;
const DIVE_COOLDOWN_SECS: f32 = 2.5;

use crate::passengers::transition;

#[derive(Component)]
pub struct Pterodactyl;

#[derive(Component)]
pub struct PteroBrain {
    pub patrol_speed: f32, // 90.0
    pub dive_speed: f32,
    pub max_force: f32,
    pub detect_radius: f32, //180.0
    pub cooldown: Timer,    // 2.5 between dives
}

impl Default for PteroBrain {
    fn default() -> Self {
        Self {
            patrol_speed: 90.0,
            dive_speed: 300.0,
            max_force: 600.0,
            detect_radius: 180.0,
            cooldown: Timer::from_seconds(2.5, TimerMode::Once), // 2.5 between dives
        }
    }
}

#[derive(Component)]
pub struct Route {
    pub points: Vec<IVec2>,
    pub next: usize,
}

#[derive(Component)]
pub struct Patrolling;

#[derive(Component)]
pub struct Telegraphing(pub Timer);

#[derive(Component)]
pub struct Diving {
    pub target: Vec2,
}

#[derive(Component)]
pub struct Recovering(pub Timer); //1.2 s

#[derive(Component, Default)]
pub struct WallContact(pub bool);

pub fn spawn_pterodactyl(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    pos: Vec2,
    route: Vec<IVec2>,
) {
    // Pre-ticked AND jittered: without this every pterodactyl in a level
    // dives in perfect synchrony the instant they all first see the player.
    /*
    let mut cooldown = Timer::from_seconds(DIVE_COOLDOWN_SECS, TimerMode::Once);
    cooldown.tick(std::time::Duration::from_secs_f32(
        rng.0.random_range(0.0..DIVE_COOLDOWN_SECS),
    ));
    */
    commands.spawn((
        PteroBrain::default(),
        Route {
            points: route,
            next: 0,
        },
        WallContact(false),
        Patrolling,
        Velocity::default(),
        Sprite::from_image(asset_server.load("sprites/ptero.png")),
        Transform::from_translation(pos.extend(z::HAZARD)),
        Hazard { radius: 16.0 },
        PhysicalTranslation(pos),
        PreviousPhysicalTranslation(pos),
        //assets.ptero.clip("flap"),
        AnimState::default(),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}

pub struct HazardPlugin;

impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                move_creatures.in_set(SimSet::Move),
                (
                    patrol_motion,
                    start_attack,
                    launch_dive,
                    dive_motion,
                    end_dive,
                    recover_motion,
                    finish_recovering,
                )
                    .chain()
                    .in_set(SimSet::Forces),
            ),
        );
        #[cfg(feature = "dev")]
        {
            app.add_systems(
                Update,
                (draw_patrol_routes, draw_raycasts, draw_attack_radius)
                    .run_if(in_state(AppState::InGame)),
            );
        }
    }
}

#[cfg(feature = "dev")]
fn draw_patrol_routes(mut gizmos: Gizmos, tile_grid: Option<Res<TileGrid>>, routes: Query<&Route>) {
    let Some(grid) = tile_grid else {
        return;
    };
    for route in &routes {
        for point in route.points.windows(2) {
            gizmos.line_2d(
                grid.tile_center(point[0]),
                grid.tile_center(point[1]),
                Color::srgb(0.5, 0.6, 0.9),
            );
        }
    }
}

#[cfg(feature = "dev")]
fn draw_raycasts(
    mut gizmos: Gizmos,
    tile_grid: Option<Res<TileGrid>>,
    player: Query<&PhysicalTranslation, With<Player>>,
    mut fliers: Query<&PhysicalTranslation, With<Patrolling>>,
) {
    let Some(grid) = tile_grid else {
        return;
    };
    let Ok(player_pos) = player.single() else {
        return;
    };
    for pos in fliers {
        let color = match grid.line_of_sight(pos.0, player_pos.0) {
            true => Color::srgb(0.2, 0.8, 0.3),
            false => Color::srgb(0.8, 0.2, 0.3),
        };
        gizmos.line_2d(pos.0, player_pos.0, color);
    }
}

fn draw_attack_radius(
    mut gizmos: Gizmos,
    mut fliers: Query<(&PhysicalTranslation, &PteroBrain), With<Patrolling>>,
) {
    for (pos, brain) in fliers {
        gizmos.circle_2d(
            Isometry2d::from_translation(pos.0),
            brain.detect_radius,
            Color::srgb(0.8, 0.3, 0.5),
        );
    }
}

// ---------------
// Systems
// ---------------
//

/// Patrolling - motion only, no transition in this system
///  'arrive' at the current waypoint; advance cyclically when close.
/// Tile -> world happens here, through TileGrid
fn patrol_motion(
    grid: Res<TileGrid>,
    time: Res<Time>,
    mut fliers: Query<
        (&PhysicalTranslation, &mut Velocity, &mut Route, &PteroBrain),
        With<Patrolling>,
    >,
) {
    let dt = time.delta_secs();
    for (pos, mut vel, mut route, brain) in &mut fliers {
        if route.points.is_empty() {
            continue;
        }
        let wp = route.points[route.next];
        let target = grid.tile_center(wp);

        let vel_0 = vel.0;
        vel.0 += steering::arrive(
            pos.0,
            vel_0,
            target,
            brain.patrol_speed,
            brain.max_force,
            brain.detect_radius,
            dt,
        );

        if pos.0.distance(target) < ARRIVE_EPS {
            route.next = (route.next + 1) % route.points.len();
        }
    }
}

// ------
// Transition owners - one system, one transition
// ------

///Patrolling -> Telegraphing: in range, in sight, cooldown ready
fn start_attack(
    time: Res<Time>,
    grid: Res<TileGrid>,
    player: Query<&PhysicalTranslation, With<Player>>,
    assets: Res<GameAssets>,
    mut fliers: Query<
        (Entity, &PhysicalTranslation, &mut Velocity, &mut PteroBrain),
        With<Patrolling>,
    >,
    mut commands: Commands,
) {
    let Ok(player_pos) = player.single() else {
        return;
    };
    for (entity, pos, mut vel, mut brain) in &mut fliers {
        brain.cooldown.tick(time.delta());
        if !brain.cooldown.is_finished() {
            continue;
        }
        if pos.0.distance(player_pos.0) > brain.detect_radius {
            continue;
        }
        if !grid.line_of_sight(pos.0, player_pos.0) {
            continue;
        }

        vel.0 *= (-8.0 * time.delta_secs()).exp();

        let bubble = spawn_bubble(&mut commands, &assets, entity, 8); // '!' the player detected!
        transition::<Patrolling>(
            &mut commands,
            entity,
            (
                Telegraphing(Timer::from_seconds(TELEGRAPH_SECS, TimerMode::Once)),
                BubbleTimer(Timer::from_seconds(1.0, TimerMode::Once)),
                Bubble(bubble),
            ),
        );
    }
}

/// Telegraphing -> Diving
fn launch_dive(
    player: Query<(&PhysicalTranslation, &Velocity), With<Player>>,
    time: Res<Time>,
    mut fliers: Query<(Entity, &mut Telegraphing)>,
    mut commands: Commands,
) {
    let Ok((player_pos, player_vel)) = player.single() else {
        return;
    };

    for (entity, mut telegraph) in &mut fliers {
        if !telegraph.0.tick(time.delta()).is_finished() {
            continue;
        }
        let target = player_pos.0 + player_vel.0 * 0.35; // pursue
        transition::<Telegraphing>(&mut commands, entity, Diving { target });
    }
}

/// While Diving: constant velocity charge
fn dive_motion(mut divers: Query<(&PhysicalTranslation, &mut Velocity, &Diving, &PteroBrain)>) {
    for (pos, mut vel, diving, brain) in &mut divers {
        vel.0 = (diving.target - pos.0).normalize_or_zero() * brain.dive_speed;
    }
}

/// Diving -> Recovering: arrived, overshoot, or thudded into a wall
fn end_dive(
    mut divers: Query<(
        Entity,
        &PhysicalTranslation,
        &Velocity,
        &Diving,
        &mut PteroBrain,
        &WallContact,
    )>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    for (entity, pos, vel, diving, mut brain, contact) in &mut divers {
        let to_target = diving.target - pos.0;
        let arrived = to_target.length() < 8.0;
        let overshot = to_target.dot(vel.0) < 0.0 && to_target.length() > 60.0;

        if arrived || overshot || contact.0 {
            //contact = hit the wall
            brain.cooldown = Timer::from_seconds(DIVE_COOLDOWN_SECS, TimerMode::Once);

            //in case of overshoot and contact show '?' bubble
            if overshot || contact.0 {
                let bubble = spawn_bubble(&mut commands, &assets, entity, 9); // should be '?' glyph
                transition::<Diving>(
                    &mut commands,
                    entity,
                    (
                        Recovering(Timer::from_seconds(RECOVER_SECS, TimerMode::Once)),
                        BubbleTimer(Timer::from_seconds(1.0, TimerMode::Once)),
                        Bubble(bubble),
                    ),
                );
            } else {
                transition::<Diving>(
                    &mut commands,
                    entity,
                    Recovering(Timer::from_seconds(RECOVER_SECS, TimerMode::Once)),
                );
            }
        }
    }
}

/// While Recovering: seek the nearest route point
fn recover_motion(
    grid: Res<TileGrid>,
    time: Res<Time>,
    mut recovering: Query<
        (&PhysicalTranslation, &mut Velocity, &Route, &PteroBrain),
        With<Recovering>,
    >,
) {
    let dt = time.delta_secs();
    for (pos, mut vel, route, brain) in &mut recovering {
        let Some(nearest) = route
            .points
            .iter()
            // check if the pterodactyl can get to a patrol point, in other words, leaving only the reachable coords.
            .filter(|p| grid.line_of_sight(grid.tile_center(**p), pos.0))
            .min_by(|a, b| {
                grid.tile_center(**a)
                    .distance_squared(pos.0)
                    .total_cmp(&grid.tile_center(**b).distance_squared(pos.0))
            })
        else {
            // let it fly
            continue;
        };
        let target = grid.tile_center(*nearest);
        let vel_0 = vel.0;
        vel.0 += steering::seek(
            pos.0,
            vel_0,
            target,
            brain.patrol_speed * 0.6,
            brain.max_force,
            dt,
        );
    }
}

/// Recovering -> Patrolling: timer done, resume from the point we recovered to.
fn finish_recovering(
    grid: Res<TileGrid>,
    time: Res<Time>,
    mut recovering: Query<(Entity, &PhysicalTranslation, &mut Recovering, &mut Route)>,
    mut commands: Commands,
) {
    for (entity, pos, mut timer, mut route) in &mut recovering {
        if !timer.0.tick(time.delta()).is_finished() {
            continue;
        }
        if let Some((idx, _)) = route.points.iter().enumerate().min_by(|(_, a), (_, b)| {
            grid.tile_center(**a)
                .distance_squared(pos.0)
                .total_cmp(&grid.tile_center(**b).distance_squared(pos.0))
        }) {
            route.next = idx;
        }
        transition::<Recovering>(&mut commands, entity, Patrolling);
    }
}

/// AI creature integrate without collision response - end_dive already owns wall-awareness via
/// its own forward probe.
fn move_creatures(
    time: Res<Time>,
    grid: Res<TileGrid>,
    mut fliers: Query<
        (
            &mut PhysicalTranslation,
            &mut PreviousPhysicalTranslation,
            &mut Velocity,
            &Hazard,
            &mut WallContact,
        ),
        With<PteroBrain>,
    >,
) {
    let dt = time.delta_secs();
    for (mut pos, mut prev, mut vel, hazard, mut contact) in &mut fliers {
        prev.0 = pos.0;
        let hazard_half = Vec2::splat(hazard.radius);
        let (new_x, hit_x) = clamp_vs_grid(&grid, hazard_half, pos.0, vel.x * dt, 0);
        if hit_x {
            vel.x = -vel.x; //0.0;
        }
        pos.0.x = new_x;

        let (new_y, hit_y) = clamp_vs_grid(&grid, hazard_half, pos.0, vel.y * dt, 1);
        if hit_y {
            vel.y = -vel.y; //0.0;
        }
        pos.0.y = new_y;

        contact.0 = hit_x || hit_y;
    }
}
