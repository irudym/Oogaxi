mod messages;
mod physics;
mod player;
mod score;
mod states;
mod zoo;

use bevy::{prelude::*, window::PresentMode};

use player::PlayerPlugin;
use states::StatesPlugin;
use zoo::ZooPlugin;

use crate::{physics::PhysicsPlugin, score::ScorePlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Oogaxi: Through the Taxiverse".into(),
                    resolution: (1280, 720).into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins((
        StatesPlugin,
        ZooPlugin,
        ScorePlugin,
        PhysicsPlugin,
        PlayerPlugin,
    ))
    .add_systems(Startup, spawn_camera);
    #[cfg(feature = "dev")]
    {
        use bevy_inspector_egui::bevy_egui::EguiPlugin;
        use bevy_inspector_egui::quick::ResourceInspectorPlugin;

        app.add_plugins(EguiPlugin::default())
            .add_plugins(ResourceInspectorPlugin::<physics::FlightConfig>::default());
    }
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
