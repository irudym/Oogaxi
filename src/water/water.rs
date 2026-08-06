use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    assets::{GameAssets, Sheet},
    camera::SceneTexture,
    colors::GameColors,
    game_rand::GameRng,
    levels::LevelOwned,
    materials::WaterMaterial,
    particles::{Fade, LifeTime, Particle},
    physics::{PhysicalTranslation, PreviousPhysicalTranslation, Velocity},
    states::AppState,
    z::z,
};

use rand::RngExt;

const SPRING_K: f32 = 0.025; // stiffness - how hard a point returns to rest
const DAMPING: f32 = 0.022; // energy loss - how fast ripples die
const SPREAD: f32 = 0.18; // neighbour coupling - how fast waves travel

#[derive(Component)]
pub struct Water;

#[derive(Component)]
pub struct WaterHighlight;

#[derive(Component)]
pub struct WaterSurface {
    pub heights: Vec<f32>, // offset from rest, per column
    pub velocities: Vec<f32>,
    pub left: f32,
    pub width: f32,
    pub rest_y: f32,
    pub depth: f32,
}

impl WaterSurface {
    pub fn new(left: f32, width: f32, rest_y: f32, depth: f32, columns: usize) -> Self {
        Self {
            heights: vec![0.0; columns],
            velocities: vec![0.0; columns],
            left,
            width,
            rest_y,
            depth,
        }
    }

    fn column_at(&self, world_x: f32) -> Option<usize> {
        if world_x < self.left || world_x > self.left + self.width {
            return None;
        }
        let t = (world_x - self.left) / self.width;
        Some(((t * (self.heights.len() - 1) as f32).round() as usize).min(self.heights.len() - 1))
    }

    pub fn splash(&mut self, world_x: f32, force: f32) {
        if let Some(i) = self.column_at(world_x) {
            self.velocities[i] += force;
        }
    }
}

/// Fixed clock: water simulation so it stays deterministic and pause freezes
pub fn simulate_water(mut surface: Query<&mut WaterSurface>) {
    for mut water in &mut surface {
        let n = water.heights.len();

        //1. each colum is a damped spring pulled toward rest (height 0)
        for i in 0..n {
            let h = water.heights[i];
            let v = water.velocities[i];
            let accel = -SPRING_K * h - DAMPING * v;
            water.velocities[i] = v + accel;
            water.heights[i] = h + water.velocities[i];
        }

        //2. coupling: each column tugs its neighbours. two passes is the standard compromise:
        // one propagation - too slow
        // three - ring
        let mut left_deltas = vec![0.0; n];
        let mut right_deltas = vec![0.0; n];
        for _ in 0..2 {
            for i in 0..n {
                if i > 0 {
                    left_deltas[i] = SPREAD * (water.heights[i] - water.heights[i - 1]);
                    water.velocities[i - 1] += left_deltas[i];
                }
                if i < n - 1 {
                    right_deltas[i] = SPREAD * (water.heights[i] - water.heights[i + 1]);
                    water.velocities[i + 1] += right_deltas[i]
                }
            }
            for i in 0..n {
                if i > 0 {
                    water.heights[i - 1] += left_deltas[i];
                }
                if i < n - 1 {
                    water.heights[i + 1] += right_deltas[i];
                }
            }
        }
    }
}

/// Two vertices per column: surface point and floor point. the strip of quads between them is the water body
fn build_water_mesh(n: usize, width: f32, depth: f32) -> Mesh {
    let heights = vec![0.0; n];
    let mut positions = Vec::with_capacity(n * 2);
    let mut uvs = Vec::with_capacity(n * 2);
    let mut colors = Vec::with_capacity(n * 2);

    for (i, h) in heights.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let x = t * width;

        positions.push([x, *h, 0.0]);
        positions.push([x, -depth, 0.0]);
        uvs.push([t, 0.0]);
        uvs.push([t, 1.0]);

        colors.push([1.0, 1.0, 1.0, 0.55]);
        colors.push([0.35, 0.45, 0.6, 0.9]); //darker below
    }

    let mut indices = Vec::with_capacity((n - 1) + 6); //6 indices per quad (consists of two triangles)
    for i in 0..(n - 1) as u32 {
        let (a, b, c, d) = (i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 3);
        indices.extend_from_slice(&[a, b, c, c, b, d]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn update_water_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    surfaces: Query<(&WaterSurface, &Mesh2d)>,
) {
    for (water, handle) in &surfaces {
        let n = water.heights.len();
        let mut positions = Vec::with_capacity(n * 2);
        for (i, h) in water.heights.iter().enumerate() {
            let x = i as f32 / (n - 1) as f32 * water.width;
            positions.push([x, *h, 0.0]);
            positions.push([x, -water.depth, 0.0]);
        }

        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
        }
    }
}

pub fn spawn_water(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<WaterMaterial>,
    assets: &GameAssets,
    scene: &SceneTexture,
    left: f32,
    width: f32,
    rest_y: f32,
    depth: f32,
) {
    // one column is 6px width
    const PX_PER_COLUMN: f32 = 6.0;
    let columns = ((width / PX_PER_COLUMN).round() as usize).clamp(8, 256);

    let mesh = meshes.add(build_water_mesh(columns, width, depth));
    let water = WaterSurface::new(left, width, rest_y, depth, columns);

    commands.spawn((
        Water,
        water,
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(WaterMaterial {
            params: Vec4::new(0.0, 0.6, 0.25, 0.4), // time, sparkle, desat, refract
            params2: Vec4::new(0.09, 0.35, 0.0, 0.0), // foam, shimmer
            noise_texture: Some(assets.get(Sheet::Water).image.clone()),
            scene: Some(scene.0.clone()),
        })),
        // Local (0,0) of the mesh sits at the left end of the resting surface.
        Transform::from_xyz(left, rest_y, z::WATER), // above the presented layer, below the lights
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}

pub fn animate_water_materials(
    water: Query<&MeshMaterial2d<WaterMaterial>, With<Water>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs_wrapped();
    for water_handle in &water {
        if let Some(mut water_mat) = water_materials.get_mut(&water_handle.0) {
            water_mat.params.x = t;
        }
    }
}

pub fn splash_on_entry(
    mut surfaces: Query<&mut WaterSurface>,
    movers: Query<(
        &PhysicalTranslation,
        &PreviousPhysicalTranslation,
        &Velocity,
    )>,
    mut commands: Commands,
    mut rng: ResMut<GameRng>,
) {
    for (pos, prev, vel) in &movers {
        for mut water in &mut surfaces {
            let surface_y = water.rest_y;
            let crossed = (prev.0.y > surface_y) != (pos.0.y > surface_y);
            if !crossed {
                continue;
            }
            let force = (vel.0.y.abs() / 10.0).clamp(0.1, 15.0);

            warn!("Splash: force: {}, vel_y: {}", force, vel.0.y.abs());

            water.splash(pos.0.x, -force);
            spawn_drops(
                &mut commands,
                &mut rng,
                Vec2::new(pos.0.x, surface_y),
                force,
            );
        }
    }
}

fn spawn_drops(commands: &mut Commands, rng: &mut GameRng, at: Vec2, intensity: f32) {
    let pos = Vec3::new(at.x, at.y, z::FX);
    let count = (8.0 + intensity * 2.0) as usize;
    for _ in 0..count {
        let side = if rng.0.random_bool(0.5) { 1.0 } else { -1.0 };
        let angle_from_horizontal: f32 = rng.0.random_range(0.1..0.8); // 6 - 46 grad
        let dir = Vec2::new(
            side * angle_from_horizontal.cos(),
            angle_from_horizontal.sin(),
        );
        let speed = rng.0.random_range(60.0..140.0) * (0.4 + intensity / 20.0);

        let vel = dir * speed;

        commands.spawn((
            Particle,
            Fade(rng.0.random_range(0.5..1.0)),
            Sprite::from_color(GameColors::WATER_DROPS, Vec2::splat(1.0)),
            Transform::from_translation(pos),
            Velocity(vel),
            LifeTime(Timer::from_seconds(
                rng.0.random_range(0.5..1.3),
                TimerMode::Once,
            )),
            DespawnOnExit(AppState::InGame),
        ));
    }
}
