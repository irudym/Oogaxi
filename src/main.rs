mod animations;
mod assets;
mod camera;
mod collision;
mod honk;
mod input;
mod integrity;
mod levels;
mod messages;
mod physics;
mod player;
mod score;
mod spritesheet;
mod states;
mod z;
mod zoo;

use bevy::{prelude::*, window::PresentMode};

use bevy_common_assets::json::JsonAssetPlugin;
use states::StatesPlugin;

use crate::animations::AnimationPlugin;
use crate::assets::AssetsPlugin;
use crate::camera::CameraPlugin;
use crate::levels::TILE;
use crate::physics::FlightConfig;
use crate::{
    collision::CollisionPlugin, input::InputPlugin, integrity::IntegrityPlugin,
    levels::LevelPlugin, physics::PhysicsPlugin, score::ScorePlugin, zoo::ZooPlugin,
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
                    resolution: (1280, 960).into(),
                    present_mode: PresentMode::AutoVsync,
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins((
        CameraPlugin,
        StatesPlugin,
        ScorePlugin,
        PhysicsPlugin,
        InputPlugin,
        LevelPlugin,
        CollisionPlugin,
        IntegrityPlugin,
        ZooPlugin,
        AnimationPlugin,
        AssetsPlugin,
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
