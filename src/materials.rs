use bevy::material::descriptor;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dKey};
use bevy_render::render_resource::{
    BlendComponent, BlendState, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};

const FLASH_SHADER: &str = "shaders/flash.wgsl";
const SCREEN_SHADER: &str = "shaders/screen_look.wgsl";
const LIGHT_COMPOSITE_SHADER: &str = "shaders/light_composite.wgsl";

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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LightCompositeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub light_map: Option<Handle<Image>>,
}

impl Material2d for LightCompositeMaterial {
    fn fragment_shader() -> ShaderRef {
        LIGHT_COMPOSITE_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(Some(target)) = fragment.targets.get_mut(0) {
                target.blend = Some(BlendState {
                    // dst = src * dst
                    color: BlendComponent {
                        src_factor: bevy_render::render_resource::BlendFactor::Dst,
                        dst_factor: bevy_render::render_resource::BlendFactor::Zero,
                        operation: bevy_render::render_resource::BlendOperation::Add,
                    },
                    // leave destination alpha alone
                    alpha: BlendComponent {
                        src_factor: bevy_render::render_resource::BlendFactor::Zero,
                        dst_factor: bevy_render::render_resource::BlendFactor::One,
                        operation: bevy_render::render_resource::BlendOperation::Add,
                    },
                });
            }
        }
        Ok(())
    }
}
