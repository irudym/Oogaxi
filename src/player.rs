use crate::{
    assets::GameAssets,
    collision::Collider,
    input::input_map_for,
    integrity::Integrity,
    levels::LevelOwned,
    physics::{
        FlightConfig, PhysicalTranslation, PreviousPhysicalTranslation, ThrustInput, Velocity,
    },
    states::AppState,
    z::z,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerId(pub u8);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // future code will be here
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    assets: &Res<GameAssets>,
    pos: Vec2,
    cfg: &FlightConfig,
) {
    let player_id: u8 = 1;
    let col_size = Vec2::new(42.0, 42.0) * 0.98 / 2.0; // TODO: need to get the frame size from atlas.
    info!(
        "Spawn player with position: {} and velocity: {:?} ",
        pos,
        Velocity::default()
    );
    // the player: a marker, sprite, position
    commands.spawn((
        Player,
        PlayerId(player_id),
        input_map_for(player_id),
        Sprite {
            image: assets.get(crate::assets::Sheet::Copter).image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.get(crate::assets::Sheet::Copter).layout.clone(),
                index: 0,
            }),
            ..default()
        },
        assets.get(crate::assets::Sheet::Copter).clip("idle"),
        crate::animations::AnimState::default(),
        Transform::from_xyz(pos.x, pos.y, z::PLAYER),
        Collider { half: col_size },
        Velocity::default(),
        ThrustInput::default(),
        PhysicalTranslation(pos),
        PreviousPhysicalTranslation(pos),
        Integrity(cfg.integrity_max),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}
