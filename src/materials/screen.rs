use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

const SCREEN_SHADER: &str = "shaders/screen_look.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ScreenMaterial {
    #[uniform(0)]
    pub params: Vec4,
}

impl Material2d for ScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        SCREEN_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
