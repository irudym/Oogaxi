use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

const FLASH_SHADER: &str = "shaders/flash.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FlashMaterial {
    #[uniform(0)]
    pub tint: LinearRgba, //color to blend toward
    #[uniform(1)]
    pub amount: f32, // 0 = untouched sprite, 1 = fully tinted
    #[uniform(2)]
    pub atlas_rect: Vec4,
    #[texture(3)]
    #[sampler(4)]
    pub sprite: Option<Handle<Image>>,
}

impl Material2d for FlashMaterial {
    fn fragment_shader() -> ShaderRef {
        FLASH_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
