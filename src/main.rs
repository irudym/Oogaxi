mod animations;
mod assets;
mod bubble;
mod camera;
mod collision;
mod colors;
mod effects;
mod game_rand;
mod hazards;
mod honk;
mod input;
mod integrity;
mod levels;
mod main_menu;
mod materials;
mod messages;
mod particles;
mod passengers;
mod physics;
mod player;
mod score;
mod spritesheet;
mod states;
mod steering;
mod z;
// mod zoo;

use bevy::sprite_render::Material2dPlugin;
use bevy::{prelude::*, window::PresentMode};

use rand::SeedableRng;
use rand::rngs::StdRng;
use states::StatesPlugin;

use crate::animations::AnimationPlugin;
use crate::assets::AssetsPlugin;
use crate::camera::CameraPlugin;
use crate::game_rand::GameRng;
use crate::hazards::HazardPlugin;
use crate::levels::TILE;
use crate::materials::FlashMaterial;
use crate::particles::ParticlesPlugin;
use crate::passengers::PassengerPlugin;
use crate::physics::FlightConfig;
use crate::{
    bubble::BubblePlugin, collision::CollisionPlugin, input::InputPlugin,
    integrity::IntegrityPlugin, levels::LevelPlugin, physics::PhysicsPlugin, score::ScorePlugin,
};

fn main() {
    // check tunneling
    let cfg = FlightConfig::default();
    debug_assert!(cfg.max_speed / 64.0 < TILE);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Oogaxi: Through the Taxiverse".into(),
                    resolution: (1280, 720).into(),
                    present_mode: PresentMode::AutoVsync,
                    resizable: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(GameRng(StdRng::seed_from_u64(0xB00)))
    .add_plugins((
        CameraPlugin,
        StatesPlugin,
        ScorePlugin,
        PhysicsPlugin,
        InputPlugin,
        LevelPlugin,
        CollisionPlugin,
        IntegrityPlugin,
        AnimationPlugin,
        AssetsPlugin,
        PassengerPlugin,
        HazardPlugin,
        BubblePlugin,
        ParticlesPlugin,
        Material2dPlugin::<FlashMaterial>::default(), //add shader
    ));
    #[cfg(feature = "dev")]
    {
        use bevy_inspector_egui::bevy_egui::EguiPlugin;
        use bevy_inspector_egui::quick::ResourceInspectorPlugin;

        app.add_plugins(EguiPlugin::default())
            .add_plugins(ResourceInspectorPlugin::<physics::FlightConfig>::default());
    }
    app.run();
}
