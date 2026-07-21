use bevy::prelude::*;

use crate::levels::TileGrid;
use crate::messages::{CopterDamaged, Landed};
use crate::physics::SimSet;
use crate::physics::{FlightConfig, PhysicalTranslation, PreviousPhysicalTranslation, Velocity};
use crate::player::Player;

/// Keep resting bodies a hair outside surface (avoid float re-penetration)
const SKIN: f32 = 0.01;

#[derive(Component, Default)]
pub struct Collider {
    pub half: Vec2,
} // half extent, 80% of sprite

#[derive(Component)]
pub struct Grounded;

#[derive(Component, Default)]
pub struct Hazard {
    pub radius: f32,
} // circle collider for lethal things

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Landed>().add_systems(
            FixedUpdate,
            (
                (move_hazards, move_and_collide).in_set(SimSet::Move),
                hazard_contact.in_set(SimSet::Contact),
            ),
        );

        #[cfg(feature = "dev")]
        app.add_systems(Update, draw_colliders);
    }
}

fn draw_colliders(
    mut gizmos: Gizmos,
    boxes: Query<(&Transform, &Collider)>,
    circles: Query<(&Transform, &Hazard)>,
) {
    for (t, c) in &boxes {
        gizmos.rect_2d(
            Isometry2d::from_translation(t.translation.truncate()),
            c.half * 2.0,
            Color::srgb(0.2, 1.0, 0.2),
        );
    }
    for (t, h) in &circles {
        gizmos.circle_2d(
            Isometry2d::from_translation(t.translation.truncate()),
            h.radius,
            Color::srgb(1.0, 0.3, 0.2),
        );
    }
}

/// Axis separated clamp of an AABB (center 'pos', 'half' extents) moving by
/// 'delta' along 'axis' (0 = x, 1 = y) against solid grid tiles.
/// Returns (allowed coordinates on that axis, hit?)
///
/// INVARIANT: |delta| < TILE, so the moved box can only newly overlap tiles
/// adjacent to its path - no cell-walking needed. Break the invariant and you
/// get tunneling
fn clamp_vs_grid(
    grid: &TileGrid,
    half: Vec2,
    mut pos: Vec2,
    delta: f32,
    axis: usize,
) -> (f32, bool) {
    let target = pos[axis] + delta;
    if delta == 0.0 {
        //not moving
        return (target, false);
    }
    pos[axis] = target; // pretend jump
    let min = pos - half; // top left corner
    let max = pos + half - Vec2::splat(0.001); // bottom right corner; half-open: don't count the far edges
    let (tx0, ty0) = grid.world_to_tile(min);
    let (tx1, ty1) = grid.world_to_tile(max);

    let mut allowed = target;
    let mut hit = false;

    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            if !grid.is_solid(tx, ty) {
                continue;
            }
            hit = true;
            let (tile_min, tile_max) = grid.tile_bounds(tx, ty);
            allowed = if delta > 0.0 {
                allowed.min(tile_min[axis] - half[axis] - SKIN)
            } else {
                allowed.max(tile_max[axis] + half[axis] + SKIN)
            };
        }
    }
    (allowed, hit)
}

/// Same contract as 'clamp_vs_grid' against dynamic AABB obstacles (platforms)
/// Returns the blocking entity so landing can name their platform
fn clamp_vs_aabbs(
    obstacles: &[(Entity, Vec2, Vec2)], // entity, center, half
    half: Vec2,
    pos: Vec2,
    delta: f32,
    axis: usize,
) -> (f32, Option<Entity>) {
    let target = pos[axis] + delta;
    if delta == 0.0 {
        return (target, None);
    }
    let other = 1 - axis;
    let mut allowed = target;
    let mut hit = None;
    for &(entity, center, ohalf) in obstacles {
        // Must overlap on the perpendicular axis to collide on this one.
        if (pos[other] - center[other]).abs() >= half[other] + ohalf[other] {
            continue;
        }
        let candidate = if delta > 0.0 {
            center[axis] - ohalf[axis] - half[axis] - SKIN
        } else {
            center[axis] + ohalf[axis] + half[axis] + SKIN
        };

        // Only obstacles actually in the way of THIS move
        let blocks = if delta > 0.0 {
            pos[axis] <= candidate && target > candidate
        } else {
            pos[axis] >= candidate && target < candidate
        };
        if blocks {
            let tighter = if delta > 0.0 {
                candidate < allowed
            } else {
                candidate > allowed
            };
            if tighter {
                allowed = candidate;
                hit = Some(entity);
            }
        }
    }
    (allowed, hit)
}

#[derive(Debug, PartialEq)]
pub enum Verdict {
    Landed,
    Crashed(f32), // severity: px/s of excess speed beyond survivable thresholds
}

/// The game's central judgment call, as pure math. 'impact' is the velocity
/// at the moment of contact - capture before any resolution zeroes it.
pub fn classify_landing(impact: Vec2, cfg: &FlightConfig) -> Verdict {
    let over_y = (impact.y.abs() - cfg.max_landing_vy).max(0.0);
    let over_x = (impact.x.abs() - cfg.max_landing_vx).max(0.0);

    if over_y == 0.0 && over_x == 0.0 {
        Verdict::Landed
    } else {
        Verdict::Crashed(over_y + over_x)
    }
}

// -----------------------
// Systems
// -----------------------

fn move_and_collide(
    cfg: Res<FlightConfig>,
    time: Res<Time>,
    grid: Res<TileGrid>,
    mut movers: Query<
        (
            Entity,
            &mut PhysicalTranslation,
            &mut PreviousPhysicalTranslation,
            &mut Velocity,
            &Collider,
            Has<Grounded>,
        ),
        With<Player>,
    >,
    mut commands: Commands,
    mut damaged: MessageWriter<CopterDamaged>,
    mut landed: MessageWriter<Landed>,
) {
    let dt = time.delta_secs();

    for (entity, mut pos, mut prev, mut vel, col, was_grounded) in &mut movers {
        prev.0 = pos.0;

        let delta = vel.0 * dt;
        // use after delta.x and delta.y

        // ----- X axis -----
        let (new_x, hit_x) = clamp_vs_grid(&grid, col.half, pos.0, delta.x, 0);
        // let (px, phit) = clamp_vs_aabbs(&plats, col.half, pos.0, dx, 0);
        // let new_x = if dx > 0.0 { gx.min(px) } else { gx.max(px) };

        //let hit_x = ghit || phit.is_some();
        if hit_x {
            let over = (vel.x.abs() - cfg.wall_crash_speed).max(0.0);
            if over > 0.0 {
                damaged.write(CopterDamaged { severity: over });
            }
            vel.x = 0.0; // scrape: slide along the wall
        }
        pos.x = new_x;

        // ----- Y axis -----
        // Capture before resolution - the classifier's whole input
        let impact = Vec2::new(vel.x, vel.y);
        let (new_y, hit_y) = clamp_vs_grid(&grid, col.half, pos.0, delta.y, 1);
        pos.y = new_y;

        if hit_y && impact.y < 0.0 {
            // bottom contact: the judgment call
            match classify_landing(impact, &cfg) {
                Verdict::Landed => {
                    vel.0 = Vec2::ZERO;
                    if !was_grounded {
                        commands.entity(entity).insert(Grounded);
                        landed.write(Landed { at: pos.0 }); // None = terrain
                    }
                }
                Verdict::Crashed(severity) => {
                    damaged.write(CopterDamaged { severity });
                    vel.0 = Vec2::ZERO;
                    if !was_grounded {
                        commands.entity(entity).insert(Grounded);
                    }
                }
            }
        } else if hit_y {
            // ceiling bonk: stop, and hard ones hurt (mirrors X-axis logic)

            let over = (impact.y.abs() - cfg.wall_crash_speed).max(0.0);
            if over > 0.0 {
                damaged.write(CopterDamaged {
                    severity: over.abs(),
                });
            }
            vel.y = 0.0;
        } else if was_grounded {
            //no vertical contact this tick and the player was grounded -> airborne:
            // thrust lifter the copter, or slides off an edge
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Hazard on the sim clock: constant velocity, bounce off solid tiles
fn move_hazards(
    time: Res<Time>,
    grid: Res<TileGrid>,
    mut hazards: Query<
        (
            &mut PhysicalTranslation,
            &mut PreviousPhysicalTranslation,
            &mut Velocity,
            &Hazard,
        ),
        Without<Player>,
    >,
) {
    let dt = time.delta_secs();
    for (mut pos, mut prev, mut vel, hazard) in &mut hazards {
        prev.0 = pos.0;
        for axis in 0..2 {
            if vel.0[axis] == 0.0 {
                continue;
            }
            let mut probe = pos.0;
            // Leading edge of the circle after this axis-move
            probe[axis] += vel.0[axis] * dt + hazard.radius.copysign(vel.0[axis]);
            let (tx, ty) = grid.world_to_tile(probe);
            if grid.is_solid(tx, ty) {
                vel.0[axis] = -vel.0[axis]; // bounce
            }
            pos.0[axis] += vel.0[axis] * dt;
        }
    }
}

/// player circle vs hazard circle. Brute force.
fn hazard_contact(
    player: Single<(&PhysicalTranslation, &Collider), With<Player>>,
    hazards: Query<(&PhysicalTranslation, &Hazard), Without<Player>>,
    cfg: Res<FlightConfig>,
    mut damaged: MessageWriter<CopterDamaged>,
) {
    let (ppos, pcol) = *player;
    let pradius = pcol.half.min_element();
    for (hpos, hazard) in &hazards {
        if ppos.0.distance_squared(hpos.0) < (pradius + hazard.radius).powi(2) {
            damaged.write(CopterDamaged {
                severity: cfg.hazard_severity,
            });
            return;
        }
    }
}

// -----------------------
// Tests
// -----------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::{TILE, TileGrid};

    fn box_room() -> TileGrid {
        TileGrid::from_ascii(&["#####", "#...#", "#...#", "#####"])
    }

    const HALF: Vec2 = Vec2::splat(12.0);

    #[test]
    fn free_move_is_untouched() {
        let grid = box_room();
        let (x, hit) = clamp_vs_grid(&grid, HALF, Vec2::ZERO, 5.0, 0);
        assert!(!hit);
        assert_eq!(x, 5.0);
    }

    #[test]
    fn stops_at_wall_from_both_sides() {
        let grid = box_room();
        // Room interior spans x in (-1.5*TILE, 1.5*TILE) — walls one tile thick.
        let wall_x = 1.5 * TILE;
        let (x, hit) = clamp_vs_grid(&grid, HALF, Vec2::ZERO, 100.0, 0);
        assert!(hit);
        assert!((x - (wall_x - HALF.x - SKIN)).abs() < 0.01, "x = {x}");

        let (x, hit) = clamp_vs_grid(&grid, HALF, Vec2::ZERO, -100.0, 0);
        assert!(hit);
        assert!((x - (-wall_x + HALF.x + SKIN)).abs() < 0.01, "x = {x}");
    }

    #[test]
    fn classify_landing_quadrants() {
        let cfg = FlightConfig::default();
        let soft = cfg.max_landing_vy * 0.5;
        let fast = cfg.max_landing_vy * 2.0;
        let straight = cfg.max_landing_vx * 0.5;
        let sliding = cfg.max_landing_vx * 2.0;

        assert!(matches!(
            classify_landing(Vec2::new(straight, -soft), &cfg),
            Verdict::Landed
        ));
        assert!(matches!(
            classify_landing(Vec2::new(straight, -fast), &cfg),
            Verdict::Crashed(_)
        ));
        assert!(matches!(
            classify_landing(Vec2::new(sliding, -soft), &cfg),
            Verdict::Crashed(_)
        ));
        assert!(matches!(
            classify_landing(Vec2::new(sliding, -fast), &cfg),
            Verdict::Crashed(_)
        ));
    }

    /// Harder impacts must yield strictly larger severities — the property
    /// the whole damage system rests on.
    #[test]
    fn severity_is_monotonic() {
        let cfg = FlightConfig::default();
        let s = |vy: f32| match classify_landing(Vec2::new(0.0, -vy), &cfg) {
            Verdict::Crashed(sev) => sev,
            Verdict::Landed => 0.0,
        };
        assert!(s(300.0) < s(400.0));
        assert!(s(400.0) < s(700.0));
        assert_eq!(s(cfg.max_landing_vy * 0.9), 0.0); // under threshold: no severity
    }
}
