use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

const REFRACTION_SHADER: &str = "shaders/refraction.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RefractionPresentMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub refraction: Option<Handle<Image>>,
}

impl Material2d for RefractionPresentMaterial {
    fn fragment_shader() -> ShaderRef {
        REFRACTION_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
