use bevy::sprite_render::Material2dPlugin;
use bevy::{prelude::*, window::PresentMode};
use oogaxi::camera::CameraPlugin;
use oogaxi::materials::*;

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
            .set(ImagePlugin::default_nearest()),
    );
    app.add_plugins((
        CameraPlugin,
        (
            Material2dPlugin::<FlashMaterial>::default(),
            Material2dPlugin::<ScreenMaterial>::default(),
            Material2dPlugin::<LightCompositeMaterial>::default(),
            Material2dPlugin::<LightMaterial>::default(),
            Material2dPlugin::<WaterMaterial>::default(),
        ), //add shaders
    ));

    app.run();
}
