/*
 * Oogaxi: Ugh! game remake 2026(C) Igor Rudym
 */
use bevy::sprite_render::Material2dPlugin;
use bevy::{prelude::*, window::PresentMode};

use oogaxi::effects::EffectsPlugin;
use oogaxi::materials::scene_present_material::ScenePresentMaterial;
use oogaxi::messages::{CopterCrashed, CopterDamaged, Landed, PassengerDelivered};
use oogaxi::overlay::OverlayPlugin;
use oogaxi::states::{AppState, StatesPlugin};
use rand::SeedableRng;
use rand::rngs::StdRng;

use oogaxi::animations::AnimationPlugin;
use oogaxi::assets::AssetsPlugin;
use oogaxi::camera::CameraPlugin;
use oogaxi::game_rand::GameRng;
use oogaxi::hazards::HazardPlugin;
use oogaxi::levels::{TILE, levels::spawn_world};
use oogaxi::lights::LightPlugin;
use oogaxi::materials::{
    FlashMaterial, LightCompositeMaterial, LightMaterial, ScreenMaterial, WaterMaterial,
};
#[cfg(feature = "dev")]
use oogaxi::overlay::OverlayCamera;
use oogaxi::particles::ParticlesPlugin;
use oogaxi::passengers::PassengerPlugin;
use oogaxi::water::WaterPlugin;
use oogaxi::{
    bubble::BubblePlugin,
    collision::CollisionPlugin,
    input::InputPlugin,
    integrity::IntegrityPlugin,
    levels::LevelPlugin,
    physics::{FlightConfig, PhysicsPlugin},
    score::ScorePlugin,
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
        (CameraPlugin, OverlayPlugin, EffectsPlugin),
        (
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
        ),
        (WaterPlugin, ParticlesPlugin, LightPlugin),
        (
            Material2dPlugin::<FlashMaterial>::default(),
            Material2dPlugin::<ScreenMaterial>::default(),
            Material2dPlugin::<LightCompositeMaterial>::default(),
            Material2dPlugin::<LightMaterial>::default(),
            Material2dPlugin::<WaterMaterial>::default(),
            Material2dPlugin::<ScenePresentMaterial>::default(),
        ),
    ))
    .add_systems(OnEnter(AppState::InGame), spawn_game);
    init_messages(&mut app);

    #[cfg(feature = "dev")]
    {
        use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin};
        use bevy_inspector_egui::quick::ResourceInspectorPlugin;

        use oogaxi::overlay::spawn_post_process;

        app.insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .add_plugins(EguiPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<FlightConfig>::default())
        .add_systems(Startup, attach_egui_to_overlay.after(spawn_post_process));
    }

    app.run();
}

#[cfg(feature = "dev")]
fn attach_egui_to_overlay(mut commands: Commands, overlay: Query<Entity, With<OverlayCamera>>) {
    if let Ok(camera) = overlay.single() {
        commands
            .entity(camera)
            .insert(bevy_inspector_egui::bevy_egui::PrimaryEguiContext);
    }
}

fn spawn_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_world(
        &mut commands,
        &asset_server,
        "levels/oogaxi.ldtk".to_string(),
    );
}

fn init_messages(app: &mut App) {
    app.add_message::<CopterCrashed>()
        .add_message::<PassengerDelivered>()
        .add_message::<Landed>()
        .add_message::<CopterDamaged>();
}
