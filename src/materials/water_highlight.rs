use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dKey};
use bevy_render::render_resource::{
    BlendComponent, BlendState, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy_render::render_resource::{BlendFactor, BlendOperation};

const HIGHLIGHT_SHADER: &str = "shaders/foam.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterHighlightMaterial {
    #[uniform(0)]
    pub params: Vec4, // x - time, y - sparkle strength, z - foam thickness, w - top line width
}

impl Material2d for WaterHighlightMaterial {
    fn fragment_shader() -> ShaderRef {
        HIGHLIGHT_SHADER.into()
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
                    color: BlendComponent {
                        src_factor: BlendFactor::One, // dst = src + dst
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                });
            }
        }
        Ok(())
    }
}
