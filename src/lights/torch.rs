use crate::lights::{Flicker, Light2d};
use crate::lights::{build_light_mesh, emissive, light_fan_mesh};
use crate::materials::LightMaterial;
use crate::{
    colors::GameColors,
    layers::LIGHT_LAYER,
    levels::{LevelOwned, TileGrid},
    states::{AppState, IsPaused},
    z::z,
};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

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
