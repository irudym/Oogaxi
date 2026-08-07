use bevy::sprite_render::Material2dPlugin;
use bevy::{prelude::*, window::PresentMode};
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use oogaxi::animations::AnimationPlugin;
use oogaxi::assets::AssetsPlugin;
use oogaxi::camera::CameraPlugin;
use oogaxi::collision::CollisionPlugin;
use oogaxi::game_rand::GameRng;
use oogaxi::levels::{LevelPlugin, levels::spawn_world};
use oogaxi::lights::LightPlugin;
use oogaxi::materials::*;
use oogaxi::overlay::OverlayPlugin;
use oogaxi::particles::ParticlesPlugin;
use oogaxi::physics::{PhysicalTranslation, PhysicsPlugin, PreviousPhysicalTranslation};
use oogaxi::player::Player;
use oogaxi::states::{AppState, StatesPlugin};
use oogaxi::water::WaterPlugin;
use rand::SeedableRng;
use rand::rngs::StdRng;

use oogaxi::overlay::{OverlayCamera, spawn_post_process};

use oogaxi::messages::{CopterCrashed, CopterDamaged, Landed, PassengerDelivered};

#[derive(Resource, Reflect)]
#[reflect(Resource)]
// WaterHighlightMaterial shader tuning
// pub params: Vec4, // // time, sparkle, desat, refract
// pub params2: Vec4, // foam, shimmer
pub struct WaterTuning {
    pub sparkle: f32,
    pub desat: f32,
    pub refract: f32,
    pub foam: f32,
    pub shimmer: f32,
}

impl Default for WaterTuning {
    fn default() -> Self {
        // params: Vec4::new(0.0, 0.6, 0.25, 0.4), // time, sparkle, desat, refract
        // params2: Vec4::new(0.09, 0.35, 0.0, 0.0), // foam, shimmer
        Self {
            sparkle: 0.6,
            desat: 0.25,
            refract: 0.4,
            foam: 0.09,
            shimmer: 0.35,
        }
    }
}

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
    app.register_type::<WaterTuning>()
        .init_resource::<WaterTuning>()
        .insert_resource(GameRng(StdRng::seed_from_u64(0xB00)))
        .add_plugins((
            (CameraPlugin, OverlayPlugin),
            (
                PhysicsPlugin,
                LevelPlugin,
                CollisionPlugin,
                WaterPlugin,
                ParticlesPlugin,
                LightPlugin,
                AnimationPlugin,
                AssetsPlugin,
                StatesPlugin, //to load assets, as it happens in AppState::Loading
            ),
            (
                Material2dPlugin::<FlashMaterial>::default(),
                Material2dPlugin::<ScreenMaterial>::default(),
                Material2dPlugin::<LightCompositeMaterial>::default(),
                Material2dPlugin::<LightMaterial>::default(),
                Material2dPlugin::<WaterMaterial>::default(),
                Material2dPlugin::<ScenePresentMaterial>::default(),
                Material2dPlugin::<RefractionPresentMaterial>::default(),
            ), //add shaders
        ))
        .add_systems(Startup, spawn_experiment_screen)
        .add_systems(FixedUpdate, reset_copter_position)
        .add_systems(Update, apply_water_shader_tuning);

    app.insert_resource(EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    })
    .add_plugins(EguiPlugin::default())
    .add_plugins(ResourceInspectorPlugin::<WaterTuning>::default())
    .add_systems(
        Update,
        attach_egui_to_overlay
            .after(spawn_post_process)
            .run_if(run_once),
    );

    init_messages(&mut app);

    app.run();
}

fn spawn_experiment_screen(
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    spawn_world(
        &mut commands,
        &asset_server,
        "levels/water_test.ldtk".to_string(),
    );
    next.set(AppState::Loading);
}

fn init_messages(app: &mut App) {
    app.add_message::<CopterCrashed>()
        .add_message::<PassengerDelivered>()
        .add_message::<Landed>()
        .add_message::<CopterDamaged>();
}

///if the copter is under water, start again
fn reset_copter_position(
    mut players: Query<(&mut PhysicalTranslation, &mut PreviousPhysicalTranslation), With<Player>>,
) {
    for (mut pos, mut prev) in &mut players {
        if pos.0.y < 24.0 {
            pos.0.y = 330.0;
            prev.0.y = 330.0;
        }
    }
}

/// Apply water tuning changes to the material
fn apply_water_shader_tuning(
    tuning: Res<WaterTuning>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    handles: Query<&MeshMaterial2d<WaterMaterial>>,
) {
    for handle in &handles {
        if let Some(mut mat) = materials.get_mut(&handle.0) {
            mat.params.y = tuning.sparkle;
            mat.params.z = tuning.desat;
            mat.params.w = tuning.refract;
            mat.params2.x = tuning.foam;
            mat.params2.y = tuning.shimmer;
        }
    }
}

fn attach_egui_to_overlay(mut commands: Commands, overlay: Query<Entity, With<OverlayCamera>>) {
    if let Ok(camera) = overlay.single() {
        commands
            .entity(camera)
            .insert(bevy_inspector_egui::bevy_egui::PrimaryEguiContext);
    }
}
