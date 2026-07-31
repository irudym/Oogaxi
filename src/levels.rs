use crate::{
    assets::GameAssets, colors::GameColors, lights::LevelAmbientColor, materials::FlashMaterial,
    physics::FlightConfig, states::IsPaused,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::states::AppState;

pub const TILE: f32 = 16.0;

/// # - solid, . - air
const TEST_CAVE: &[&str] = &[
    "####################",
    "#..................#",
    "#......######......#",
    "#..................#",
    "#..###..........##.#",
    "#..................#",
    "#.......####.......#",
    "#..................#",
    "####################",
];

#[derive(Component)]
pub struct LevelOwned;

#[derive(Resource)]
pub struct TileGrid {
    rows: i32,
    cols: i32,
    solid: Vec<bool>,
    origin: Vec2,
}

#[derive(Resource)]
pub struct DayTime(Timer);

#[derive(Resource, Default)]
pub struct AnimateAmbient(bool);

impl Default for DayTime {
    fn default() -> Self {
        Self(Timer::from_seconds(120.0, TimerMode::Repeating))
    }
}

impl TileGrid {
    pub fn from_ascii(level: &[&str]) -> Self {
        let rows = level.len() as i32;
        let cols = level[0].len() as i32;

        let mut solid = vec![false; (rows * cols) as usize];
        for (level_row, line) in level.iter().enumerate() {
            debug_assert_eq!(line.len() as i32, cols, "ragged test_cave row");
            // level row 0 is top; world row 0 is bottom -> flip
            let world_row = rows as usize - level_row - 1;
            for (col, ch) in line.chars().enumerate() {
                solid[world_row * cols as usize + col] = ch == '#';
            }
        }

        // center the cave
        let size = Vec2::new(cols as f32, rows as f32) * TILE;

        Self {
            rows,
            cols,
            solid,
            origin: -size / 2.0,
        }
    }
    pub fn is_solid(&self, tile_x: i32, tile_y: i32) -> bool {
        if tile_x < 0 || tile_x >= self.cols || tile_y < 0 || tile_y >= self.rows {
            return true;
        }
        self.solid[(tile_y * self.cols + tile_x) as usize]
    }

    pub fn world_to_tile(&self, p: Vec2) -> (i32, i32) {
        let new_p = (p - self.origin) / TILE;
        (new_p.x.floor() as i32, new_p.y.floor() as i32)
    }

    /// World-space (min, max) corner of the tile
    pub fn tile_bounds(&self, tile_x: i32, tile_y: i32) -> (Vec2, Vec2) {
        let min = self.origin + Vec2::new(tile_x as f32, tile_y as f32) * TILE;
        (min, min + Vec2::splat(TILE))
    }

    pub fn from_wall_coord(
        cols: i32,
        rows: i32,
        origin: Vec2,
        walls: impl Iterator<Item = GridCoords>,
    ) -> Self {
        let mut solid = vec![false; (rows * cols) as usize];
        for c in walls {
            if c.x >= 0 && c.y >= 0 && c.x < cols && c.y < rows {
                solid[(c.y * cols + c.x) as usize] = true;
            }
        }
        Self {
            cols,
            rows,
            solid,
            origin,
        }
    }

    /// True, if the segment from 'a' to 'b' crosses no solid cell
    pub fn line_of_sight(&self, a: Vec2, b: Vec2) -> bool {
        let (mut tx, mut ty) = self.world_to_tile(a);
        let (ex, ey) = self.world_to_tile(b);

        let d = b - a;
        let step_x: i32 = if d.x > 0.0 { 1 } else { -1 };
        let step_y: i32 = if d.y > 0.0 { 1 } else { -1 };

        //t to the first x/y boundary, and t per full cell:
        let (mut t_max_x, t_delta_x) = axis_params(a.x, d.x, self.origin.x, tx, step_x);
        let (mut t_max_y, t_delta_y) = axis_params(a.y, d.y, self.origin.y, ty, step_y);
        loop {
            if self.is_solid(tx, ty) {
                return false;
            }
            if tx == ex && ty == ey {
                return true;
            }
            if t_max_x < t_max_y {
                tx += step_x;
                t_max_x += t_delta_x;
            } else {
                ty += step_y;
                t_max_y += t_delta_y;
            }
        }
    }

    pub fn tile_center(&self, tile_coord: IVec2) -> Vec2 {
        let (min, max) = self.tile_bounds(tile_coord.x, tile_coord.y);
        (min + max) / 2.0
    }
}

fn axis_params(a: f32, d: f32, origin: f32, tile: i32, step: i32) -> (f32, f32) {
    if d == 0.0 {
        return (f32::INFINITY, f32::INFINITY); //parallel: never crosses this axis
    }
    let next_boundary = origin + (tile + step.max(0)) as f32 * TILE;
    ((next_boundary - a) / d, (TILE / d).abs())
}

#[derive(Debug)]
pub struct Stop {
    pub address: u8,
    pub sign_pos: Vec2, // captured from TaxiStop entity's GlobalTransform, where passengers wait (sign tile - 1)
    pub cave_pos: Vec2, // captured from CaveEntrance entity's GlobalTransform, where passenger enter and leave the world
}

#[derive(Resource, Default)]
pub struct TaxiRegistry(pub Vec<Stop>);

impl TaxiRegistry {
    pub fn by_address(&self, a: u8) -> Option<&Stop> {
        self.0.iter().find(|s| s.address == a)
    }

    pub fn stop_near(&self, pos: Vec2, radius: f32) -> Option<&Stop> {
        self.0
            .iter()
            .map(|s| (s, s.sign_pos.distance_squared(pos)))
            .filter(|(_, d2)| *d2 <= radius * radius)
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(s, _)| s)
    }

    pub fn fare_between(&self, from: u8, to: u8, cfg: &FlightConfig) -> f32 {
        match (self.by_address(from), self.by_address(to)) {
            (Some(a), Some(b)) => cfg.fare_base + a.sign_pos.distance(b.sign_pos) * cfg.fare_per_px,
            _ => cfg.fare_min,
        }
    }
}

#[derive(Default, Component)]
pub struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .insert_resource(LevelSelection::index(0))
            .insert_resource(DayTime::default())
            .insert_resource(AnimateAmbient::default())
            .register_ldtk_int_cell::<WallBundle>(1) //1  - wall type in LDtk
            .add_systems(OnEnter(AppState::InGame), spawn_world)
            .add_systems(
                PostUpdate,
                (despawn_level_owned, convert_editor_entities)
                    .chain()
                    .after(TransformSystems::Propagate)
                    .run_if(resource_exists::<GameAssets>),
            )
            .add_systems(
                Update,
                (
                    rebuild_tile_grid,
                    dev_level_keys,
                    tick_daytime_clock.run_if(in_state(IsPaused::Running)),
                    update_level_ambient
                        .run_if(resource_changed::<DayTime>)
                        .run_if(|a: Res<AnimateAmbient>| a.0),
                ),
            )
            .add_systems(OnExit(AppState::InGame), remove_tile_grid);
    }
}

fn tick_daytime_clock(time: Res<Time>, mut clock: ResMut<DayTime>) {
    clock.0.tick(time.delta());
}

fn color_lerp(color_1: Color, color_2: Color, fact: f32) -> Color {
    let c1 = color_1.to_linear();
    let c2 = color_2.to_linear();

    Color::LinearRgba(LinearRgba {
        red: c1.red.lerp(c2.red, fact),
        green: c1.green.lerp(c2.green, fact),
        blue: c1.blue.lerp(c2.blue, fact),
        alpha: c1.alpha.lerp(c2.alpha, fact),
    })
}

fn update_level_ambient(mut ambient: ResMut<LevelAmbientColor>, clock: Res<DayTime>) {
    // TODO: need to make a distinguish day - time periods and make MORNING and EVENING time frames smaller
    let fract = clock.0.fraction();
    let color = if fract < 0.25 {
        color_lerp(
            GameColors::NIGHT_AMBIENT,
            GameColors::MORNING_AMBIENT,
            fract / 0.25,
        )
    } else if fract >= 0.25 && fract < 0.5 {
        color_lerp(
            GameColors::MORNING_AMBIENT,
            GameColors::DAY_AMBIENT,
            (fract - 0.25) / 0.25,
        )
    } else if fract >= 0.5 && fract < 0.75 {
        color_lerp(
            GameColors::DAY_AMBIENT,
            GameColors::EVENING_AMBIENT,
            (fract - 0.5) / 0.25,
        )
    } else {
        color_lerp(
            GameColors::EVENING_AMBIENT,
            GameColors::NIGHT_AMBIENT,
            (fract - 0.75) / 0.25,
        )
    };
    ambient.0 = color;
}

/// OnExit(AppState::InGame): the level is gone, so the grid built from it
/// must not survive to lie to the next session's simulation.
fn remove_tile_grid(mut commands: Commands) {
    commands.remove_resource::<TileGrid>();
}

fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: asset_server.load("levels/oogaxi.ldtk").into(),
            ..Default::default()
        },
        DespawnOnExit(AppState::InGame),
    ));
}

/// When a level finished spawning, harvest its Wall cells into our TileGrid
/// collision.rs consumes the same resources it always did
fn rebuild_tile_grid(
    mut level_events: MessageReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    mut commands: Commands,
) {
    for event in level_events.read() {
        let LevelEvent::Spawned(level_id) = event else {
            continue;
        };

        let project = project_assets
            .get(projects.single().expect("one LDtk world"))
            .expect("project loaded if level spawned");
        let level = project
            .get_raw_level_by_iid(level_id.get())
            .expect("spawned level exists in project");

        let ambient = level
            .get_color_field("Ambient")
            .copied()
            .unwrap_or(LevelAmbientColor::default().0);
        commands.insert_resource(LevelAmbientColor(ambient));

        let animate_ambient = level.get_bool_field("DayNight").copied().unwrap_or(false);
        commands.insert_resource(AnimateAmbient(animate_ambient));

        let cols = level.px_wid / TILE as i32;
        let rows = level.px_hei / TILE as i32;
        let origin = Vec2::ZERO;

        commands.insert_resource(TileGrid::from_wall_coord(
            cols,
            rows,
            origin,
            walls.iter().copied(),
        ));

        commands.insert_resource(crate::camera::LevelBounds {
            min: Vec2::ZERO,
            max: Vec2::new(level.px_wid as f32, level.px_hei as f32),
        });
    }
}

fn dev_level_keys(keys: Res<ButtonInput<KeyCode>>, mut selection: ResMut<LevelSelection>) {
    let LevelSelection::Indices(indices) = &mut *selection else {
        return;
    };

    if keys.just_pressed(KeyCode::BracketRight) {
        indices.level += 1;
    }
    if keys.just_pressed(KeyCode::BracketLeft) && indices.level > 0 {
        indices.level -= 1;
    }
}

fn convert_editor_entities(
    mut commands: Commands,
    instances: Query<(Entity, &EntityInstance, &GlobalTransform), Added<EntityInstance>>,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    cfg: Res<FlightConfig>,
    layers: Query<&LayerMetadata>,
    parents: Query<&ChildOf>,
    grid: Option<Res<TileGrid>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FlashMaterial>>,
    mut color_mat: ResMut<Assets<ColorMaterial>>,
) {
    let Some(grid) = grid else {
        return;
    };
    if instances.is_empty() {
        return;
    }

    // PASS 1: platforms
    // PASS 2: taxi infrastructure
    let mut signs: Vec<(u8, Vec2)> = Vec::new();
    let mut caves: Vec<(u8, Vec2)> = Vec::new();
    for (_, inst, transform) in &instances {
        let pos = transform.translation().truncate();
        match inst.identifier.as_str() {
            "TaxiStop" => {
                let Ok(addr) = inst.get_int_field("address") else {
                    warn!("TaxiStop at {pos} missing 'address' - skipped");
                    continue;
                };
                signs.push((*addr as u8, pos));
                crate::passengers::spawn_sign(&mut commands, &assets, pos, *addr as u8);
            }
            "CaveEntrance" => {
                let Ok(addr) = inst.get_int_field("address") else {
                    warn!("CaveEntrance at {pos} missing 'address' - skipped");
                    continue;
                };
                caves.push((*addr as u8, pos));
                crate::passengers::spawn_cave(&mut commands, &assets, pos);
            }
            "Torch" => {
                crate::lights::spawn_torch(&mut commands, &mut meshes, &mut color_mat, &grid, pos);
            }
            _ => {}
        }
    }

    // Pair by address
    let mut registry = TaxiRegistry::default();
    for (address, sign_pos) in &signs {
        let Some((_, cave_pos)) = caves.iter().find(|(a, _)| a == address) else {
            warn!("TaxiStop '{address}' found no CaveEntrance with the same address - skipped");
            continue;
        };
        registry.0.push(Stop {
            address: *address,
            sign_pos: *sign_pos,
            cave_pos: *cave_pos,
        });
    }

    if !registry.0.is_empty() {
        info!(
            "taxi registry: {} stops, walks of {:?} px",
            registry.0.len(),
            registry
                .0
                .iter()
                .map(|s| (s.cave_pos.x - s.sign_pos.x).abs().round())
                .collect::<Vec<_>>()
        );
    }

    commands.insert_resource(registry);

    for (editor_entity, inst, transform) in &instances {
        let pos = transform.translation().truncate();
        // Walk up to the parent layer to learn its height in tiles:
        let grid_height = parents
            .get(editor_entity)
            .ok()
            .and_then(|p| layers.get(p.parent()).ok())
            .map(|l| l.c_hei) // LDtk's own field name for grid rows
            .unwrap_or(0);
        match inst.identifier.as_str() {
            "TaxiStop" | "CaveEntrance" => {} // consumed above
            "PlayerSpawn" => crate::player::spawn_player(
                &mut commands,
                &assets,
                pos,
                &cfg,
                &mut meshes,
                &mut materials,
            ),
            "Pterodactyl" => {
                let route: Vec<IVec2> = inst
                    .get_maybe_points_field("patrol")
                    .map(|pts| {
                        pts.iter()
                            .flatten()
                            // y flip
                            .map(|p| IVec2::new(p.x, grid_height - p.y - 1))
                            .collect()
                    })
                    .unwrap_or_default();

                for pair in route.windows(2) {
                    let a = grid.tile_center(pair[0]);
                    let b = grid.tile_center(pair[1]);
                    if !grid.line_of_sight(a, b) {
                        warn!(
                            "The route has a blind corners between {:?} and {:?} - will get stuck!",
                            pair[0], pair[1]
                        );
                    }
                }
                if route.len() < 2 {
                    warn!(
                        "Pterodactyl at {pos} has a route of {} points, needs 2+; skipped",
                        route.len()
                    )
                } else {
                    crate::hazards::spawn_pterodactyl(&mut commands, &asset_server, pos, route);
                }
            }
            other => warn!("LDtk entity with no converter: {other} "),
        }
        commands.entity(editor_entity).despawn();
    }
}

///When a level (re)spawn begins, everything owned by old level goes
fn despawn_level_owned(
    mut level_events: MessageReader<LevelEvent>,
    owned: Query<Entity, With<LevelOwned>>,
    mut commands: Commands,
) {
    for event in level_events.read() {
        if matches!(event, LevelEvent::SpawnTriggered(_)) {
            for entity in &owned {
                commands.entity(entity).despawn();
            }
        }
    }
}
