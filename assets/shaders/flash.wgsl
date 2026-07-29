#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> tint: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> amount: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> atlas_rect: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sprite_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sprite_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // mesh.uv is 0..1 across the quad
    let frame_uv = atlas_rect.zw + mesh.uv * atlas_rect.xy;
    let base = textureSample(sprite_texture, sprite_sampler, frame_uv);

    // Blend toward the tint, but PRESERVE the sprite's alpha - otherwise
    // the flash fills the transparent parts of the quad and will produce
    // a glowing rectangle instead of glowing sprite
    let flashed = mix(base.rgb, tint.rgb, amount);
    return vec4<f32>(flashed, base.a);
}
