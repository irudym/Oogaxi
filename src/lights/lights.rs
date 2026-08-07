use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::TextureFormat;
use bevy::{
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    prelude::*,
};
use rand::RngExt;

use crate::camera::projection::virtual_projection;
use crate::camera::tracking::TracksGameCamera;
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
pub struct LightCamera;

#[derive(Resource)]
pub struct LevelAmbientColor(pub Color);

impl Default for LevelAmbientColor {
    fn default() -> Self {
        Self(Color::srgb(0.60, 0.44, 0.56)) // cave default
    }
}

pub fn emissive(color: Color, strength: f32) -> Color {
    let c = color.to_linear();
    Color::LinearRgba(LinearRgba {
        red: c.red * strength,
        green: c.green * strength,
        blue: c.blue * strength,
        alpha: c.alpha,
    })
}

pub fn setup_light_map(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
        TracksGameCamera,
        Camera2d,
        LightCamera,
        Camera {
            order: -1,                           // before the game camera, the map should exist first
            clear_color: ClearColorConfig::None, //ClearColorConfig::Custom(Color::srgb(0.75, 0.66, 0.90)),
            ..default()
        },
        RenderTarget::Image(handle.clone().into()),
        RenderLayers::layer(LIGHT_LAYER),
        virtual_projection(),
    ));
    commands.insert_resource(LightMap(handle));
}

pub fn apply_ambient(
    ambient: Res<LevelAmbientColor>,
    mut light_cam: Query<&mut Camera, With<LightCamera>>,
) {
    let Ok(mut camera) = light_cam.single_mut() else {
        return;
    };
    camera.clear_color = ClearColorConfig::Custom(ambient.0);
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

pub fn flicker_lights(
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
pub fn build_light_mesh(grid: &TileGrid, origin: Vec2, radius: f32, rays: usize) -> Vec<Vec2> {
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
pub fn light_fan_mesh(origin: Vec2, rim: &[Vec2]) -> Mesh {
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
