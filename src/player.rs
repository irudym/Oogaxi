use crate::{
    assets::GameAssets,
    collision::Collider,
    input::input_map_for,
    integrity::Integrity,
    levels::LevelOwned,
    materials::FlashMaterial,
    physics::{
        FlightConfig, PhysicalTranslation, PreviousPhysicalTranslation, ThrustInput, Velocity,
    },
    states::AppState,
    z::z,
};
use bevy::prelude::*;

use crate::assets::Sheet;

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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<FlashMaterial>,
) {
    let sheet = &assets.get(Sheet::Copter);
    let (idle_frame, _) = sheet.clip("idle").frames[0];

    let frame_size = sheet.get_frame_size(idle_frame); // TODO: disaster in case of different frame sizes

    let player_id: u8 = 1;
    let col_size = Vec2::new(frame_size.x as f32, frame_size.y as f32) * 0.98 / 2.0; // TODO: need to get the frame size from atlas.
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
        (
            Mesh2d(meshes.add(Rectangle::new(frame_size.x as f32, frame_size.y as f32))),
            MeshMaterial2d(materials.add(FlashMaterial {
                tint: LinearRgba::WHITE,
                amount: 0.0,
                atlas_rect: sheet.atlas_rect(idle_frame),
                sprite: Some(sheet.image.clone()),
            })),
        ),
        sheet.clip("idle"),
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
