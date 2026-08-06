use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::TextureFormat;
use bevy::{
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    prelude::*,
};
use rand::RngExt;

use crate::materials::LightMaterial;
use crate::{
    camera::{GameCamera, camera_follow},
    colors::GameColors,
    game_rand::GameRng,
    layers::LIGHT_LAYER,
    levels::{LevelOwned, TileGrid},
    states::{AppState, IsPaused},
    z::z,
};

#[derive(Resource)]
pub struct LightMap(pub Handle<Image>);

#[derive(Component)]
struct LightCamera;

#[derive(Resource)]
pub struct LevelAmbientColor(pub Color);

impl Default for LevelAmbientColor {
    fn default() -> Self {
        Self(Color::srgb(0.60, 0.44, 0.56)) // cave default
    }
}

fn emissive(color: Color, strength: f32) -> Color {
    let c = color.to_linear();
    Color::LinearRgba(LinearRgba {
        red: c.red * strength,
        green: c.green * strength,
        blue: c.blue * strength,
        alpha: c.alpha,
    })
}

fn setup_light_map(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Half resolution is enough;
    let image = Image::new_target_texture(
        320,
        180,
        TextureFormat::Rgba8UnormSrgb,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    let handle = images.add(image);

    // The light camera renders only the light layer, into the texture
    commands.spawn((
        Camera2d,
        LightCamera,
        Camera {
            order: -1, // before the game camera, the map should exist first
            clear_color: ClearColorConfig::Custom(Color::srgb(0.75, 0.66, 0.90)),
            ..default()
        },
        RenderTarget::Image(handle.clone().into()),
        RenderLayers::layer(LIGHT_LAYER),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 640.0,
                height: 360.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.insert_resource(LightMap(handle));
}

/// Follow the game camera
fn sync_light_camera(
    game: Query<&Transform, (With<GameCamera>, Without<LightCamera>)>,
    mut light: Query<&mut Transform, With<LightCamera>>,
) {
    let (Ok(game_tf), Ok(mut light_tf)) = (game.single(), light.single_mut()) else {
        return;
    };
    *light_tf = *game_tf;
}

fn apply_ambient(
    ambient: Res<LevelAmbientColor>,
    mut light_cam: Query<&mut Camera, With<LightCamera>>,
) {
    let Ok(mut camera) = light_cam.single_mut() else {
        return;
    };
    camera.clear_color = ClearColorConfig::Custom(ambient.0);
}

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelAmbientColor::default())
            .add_systems(Startup, setup_light_map)
            .add_systems(
                Update,
                (
                    sync_light_camera.after(camera_follow),
                    flicker_lights.run_if(in_state(IsPaused::Running)),
                    apply_ambient.run_if(resource_changed::<LevelAmbientColor>),
                ),
            );
    }
}

#[derive(Component)]
pub struct Light2d {
    pub radius: f32,
    pub color: Color,
    pub intensity: f32,
}

#[derive(Component)]
pub struct Flicker {
    pub phase: f32,
}

fn flicker_lights(
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    mut materials: ResMut<Assets<LightMaterial>>,
    mut lights: Query<(&MeshMaterial2d<LightMaterial>, &mut Flicker, &Light2d)>,
) {
    for (handle, mut flicker, light) in &mut lights {
        let target = rng.0.random_range(0.82..1.0);
        let alpha = 1.0 - (-9.0 * time.delta_secs()).exp();
        flicker.phase += (target - flicker.phase) * alpha;
        if let Some(mut mat) = materials.get_mut(&handle.0) {
            mat.color = light
                .color
                .with_alpha(flicker.phase * light.intensity)
                .into();
        }
    }
}

/// Cant N rays outward; where LOS fails, shorten the ray to the hit point.
/// The result: the lit area. 32 rays per 96px torch.
fn build_light_mesh(grid: &TileGrid, origin: Vec2, radius: f32, rays: usize) -> Vec<Vec2> {
    (0..rays)
        .map(|i| {
            let angle = i as f32 / rays as f32 * std::f32::consts::TAU;
            let dir = Vec2::from_angle(angle);
            let far = origin + dir * radius;
            if grid.line_of_sight(origin, far) {
                far
            } else {
                // binary-search the boundary - 6 iteration is sub-pixel at 96px
                let mut lo = 0.0;
                let mut hi = radius;
                for _ in 0..6 {
                    let mid = (lo + hi) * 0.5;
                    if grid.line_of_sight(origin, origin + dir * mid) {
                        lo = mid
                    } else {
                        hi = mid
                    }
                }
                origin + dir * lo
            }
        })
        .collect()
}

/// Visibility fan -> mesh
/// Bright at the center, transparent at the rim: vertex color do the falloff
fn light_fan_mesh(origin: Vec2, rim: &[Vec2]) -> Mesh {
    let n = rim.len();
    let mut positions = Vec::with_capacity(n + 1);
    let mut colors = Vec::with_capacity(n + 1);

    positions.push([0.0, 0.0, 0.0]);
    colors.push([1.0, 1.0, 1.0, 1.0]);

    for p in rim {
        let local = *p - origin; // world to local
        positions.push([local.x, local.y, 0.0]);
        colors.push([1.0, 1.0, 1.0, 0.0]); // transparent at the rim
    }

    // one triangle per segment
    let mut indices = Vec::with_capacity(n * 3);
    for i in 0..n {
        indices.push(0u32);
        indices.push(1 + i as u32);
        indices.push(1 + ((i + 1) % n) as u32);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

#[derive(Component)]
struct Torch;

pub fn spawn_torch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<LightMaterial>,
    grid: &TileGrid,
    pos: Vec2,
) {
    commands.spawn((
        Torch,
        Sprite::from_color(emissive(GameColors::TORCH_CORE, 5.0), Vec2::splat(4.0)),
        Transform::from_translation(pos.extend(z::LIGHTS)),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));

    let radius = 96.0;
    let rim = build_light_mesh(grid, pos, radius, 32);

    // The Light on the light layer, soft radial gradient sprite
    // TODO: Should be pixel art style texture

    commands.spawn((
        Light2d {
            radius,
            color: GameColors::TORCH_FLAME,
            intensity: 0.6,
        },
        Flicker { phase: 0.0 },
        Mesh2d(meshes.add(light_fan_mesh(pos, &rim))),
        MeshMaterial2d(materials.add(LightMaterial {
            color: GameColors::TORCH_FLAME.into(),
        })),
        Transform::from_translation(pos.extend(0.0)),
        RenderLayers::layer(LIGHT_LAYER),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}
