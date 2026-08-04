use bevy::sprite_render::Material2dPlugin;
use bevy::{prelude::*, window::PresentMode};
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use oogaxi::camera::CameraPlugin;
use oogaxi::collision::CollisionPlugin;
use oogaxi::game_rand::GameRng;
use oogaxi::levels::{LevelPlugin, spawn_world};
use oogaxi::lights::LightPlugin;
use oogaxi::materials::*;
use oogaxi::particles::ParticlesPlugin;
use oogaxi::physics::PhysicsPlugin;
use oogaxi::states::{AppState, IsPaused};
use oogaxi::water::WaterPlugin;
use rand::SeedableRng;
use rand::rngs::StdRng;

use oogaxi::messages::{CopterCrashed, CopterDamaged, Landed, PassengerDelivered};

fn main() {
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
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                file_path: "../../assets".to_string(),
                ..Default::default()
            }),
    );
    app.insert_resource(GameRng(StdRng::seed_from_u64(0xB00)))
        .init_state::<AppState>()
        .add_sub_state::<IsPaused>()
        .add_plugins((
            CameraPlugin,
            PhysicsPlugin,
            LevelPlugin,
            CollisionPlugin,
            WaterPlugin,
            ParticlesPlugin,
            LightPlugin,
            (
                Material2dPlugin::<FlashMaterial>::default(),
                Material2dPlugin::<ScreenMaterial>::default(),
                Material2dPlugin::<LightCompositeMaterial>::default(),
                Material2dPlugin::<LightMaterial>::default(),
                Material2dPlugin::<WaterMaterial>::default(),
            ), //add shaders
        ))
        .add_systems(Startup, spawn_experiment_screen);

    app.insert_resource(EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    })
    .add_plugins(EguiPlugin::default());
    //.add_plugins(ResourceInspectorPlugin::<FlightConfig>::default());
    //

    init_messages(&mut app);

    app.run();
}

fn spawn_experiment_screen(
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
    mut next_pause: ResMut<NextState<IsPaused>>,
    asset_server: Res<AssetServer>,
) {
    next.set(AppState::InGame);
    next_pause.set(IsPaused::Running);
    spawn_world(
        &mut commands,
        &asset_server,
        "levels/water_test.ldtk".to_string(),
    );
}

fn init_messages(app: &mut App) {
    app.add_message::<CopterCrashed>()
        .add_message::<PassengerDelivered>()
        .add_message::<Landed>()
        .add_message::<CopterDamaged>();
}
