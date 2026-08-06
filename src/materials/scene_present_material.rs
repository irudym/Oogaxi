use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

const SCENE_SHADER: &str = "shaders/scene.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ScenePresentMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub scene: Option<Handle<Image>>,
}

impl Material2d for ScenePresentMaterial {
    fn fragment_shader() -> ShaderRef {
        SCENE_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
