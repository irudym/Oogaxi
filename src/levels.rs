use bevy::prelude::*;

use crate::states::AppState;

pub const TILE: f32 = 64.0;

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

#[derive(Resource)]
pub struct TileGrid {
    rows: i32,
    cols: i32,
    solid: Vec<bool>,
    origin: Vec2,
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
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileGrid::from_ascii(TEST_CAVE))
            .add_systems(OnEnter(AppState::InGame), spawn_cave_visuals);
    }
}

fn spawn_cave_visuals(mut commands: Commands, grid: Res<TileGrid>) {
    for ty in 0..grid.rows {
        for tx in 0..grid.cols {
            if !grid.is_solid(tx, ty) {
                continue;
            }

            let (min, _) = grid.tile_bounds(tx, ty);
            commands.spawn((
                Sprite::from_color(Color::srgb(0.35, 0.32, 0.28), Vec2::splat(TILE)),
                Transform::from_translation((min + Vec2::splat(TILE / 2.0)).extend(0.0)),
                DespawnOnExit(AppState::InGame),
            ));
        }
    }
}
