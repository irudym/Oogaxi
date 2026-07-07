use bevy::{prelude::*, window::PresentMode};

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2D Camera
    commands.spawn(Camera2d);

    // the player: a marker, sprite, position
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("sprites/copter.png")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn follow_mouse(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let (camera, camera_transform) = camera.into_inner();

    if let Some(cursor_pos) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos)
    {
        player.translation = player.translation.lerp(
            world_pos.extend(player.translation.z),
            1.0 - (-1.0 * time.delta_secs()).exp(),
        );
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Oogaxi: Taxiverse".into(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, follow_mouse)
        .run();
}
