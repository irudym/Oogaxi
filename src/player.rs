use crate::{
    physics::{PhysicalTranslation, PreviousPhysicalTranslation, ThrustInput, Velocity},
    states::{AppState, IsPaused},
};
use bevy::prelude::*;

#[derive(Component)]
struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_player);
    }
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start = Vec2::new(0.0, 0.0);
    // the player: a marker, sprite, position
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("sprites/copter.png")),
        Transform::from_xyz(start.x, start.y, 0.0),
        Velocity::default(),
        ThrustInput::default(),
        PhysicalTranslation(start),
        PreviousPhysicalTranslation(start),
        DespawnOnExit(AppState::InGame),
    ));
}
