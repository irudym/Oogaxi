use crate::{
    collision::Collider,
    input::input_map_for,
    integrity::Integrity,
    physics::{PhysicalTranslation, PreviousPhysicalTranslation, ThrustInput, Velocity},
    states::AppState,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerId(pub u8);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_player);
    }
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start = Vec2::new(0.0, 0.0);
    let player_id: u8 = 1;
    // the player: a marker, sprite, position
    commands.spawn((
        Player,
        PlayerId(player_id),
        input_map_for(player_id),
        Sprite::from_image(asset_server.load("sprites/copter_32.png")),
        Transform::from_xyz(start.x, start.y, 0.0),
        Collider {
            half: Vec2::new(14.0, 16.0),
        },
        Velocity::default(),
        ThrustInput::default(),
        PhysicalTranslation(start),
        PreviousPhysicalTranslation(start),
        Integrity(100.0),
        DespawnOnExit(AppState::InGame),
    ));
}
