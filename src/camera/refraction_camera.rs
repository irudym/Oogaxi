use bevy::camera::RenderTarget;
use bevy::render::render_resource::TextureFormat;
use bevy::{
    camera::{Hdr, visibility::RenderLayers},
    prelude::*,
};

use crate::{
    camera::{
        projection::{VIRTUAL_RESOLUTION, virtual_projection},
        tracking::TracksGameCamera,
    },
    layers::REFRACTION_LAYER,
};

#[derive(Component)]
pub struct RefractionCamera;

#[derive(Resource)]
pub struct RefractionTexture(pub Handle<Image>);

pub fn spawn_refraction_camera(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
) -> Handle<Image> {
    // create image texture to apply future shaders
    let image = images.add(Image::new_target_texture(
        VIRTUAL_RESOLUTION.x as u32,
        VIRTUAL_RESOLUTION.y as u32,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));

    commands.spawn((
        TracksGameCamera, //sync with Game Camera movements
        Camera2d,
        RefractionCamera,
        Hdr,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        virtual_projection(),
        RenderTarget::Image(image.clone().into()),
        RenderLayers::layer(REFRACTION_LAYER),
    ));
    commands.insert_resource(RefractionTexture(image.clone()));

    image
}
