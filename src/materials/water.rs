use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

const WATER_SHADER: &str = "shaders/water.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub params: Vec4, // // time, sparkle, desat, refract
    #[uniform(1)]
    pub params2: Vec4, // foam, shimmer
    #[texture(2)]
    #[sampler(3)]
    pub scene: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub noise_texture: Option<Handle<Image>>,
}

impl Material2d for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        WATER_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
