use crate::{
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
    asset_server: &Res<AssetServer>,
    pos: Vec2,
    cfg: &FlightConfig,
) {
    let player_id: u8 = 1;
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
        Sprite::from_image(asset_server.load("sprites/copter_32.png")),
        Transform::from_xyz(pos.x, pos.y, z::PLAYER),
        Collider {
            half: Vec2::new(14.0, 16.0),
        },
        Velocity::default(),
        ThrustInput::default(),
        PhysicalTranslation(pos),
        PreviousPhysicalTranslation(pos),
        Integrity(cfg.integrity_max),
        LevelOwned,
        DespawnOnExit(AppState::InGame),
    ));
}
