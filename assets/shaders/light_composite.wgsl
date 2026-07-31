#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var light_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var light_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let light = textureSample(light_map, light_sampler, mesh.uv).rgb;

    return vec4<f32>(light, 1.0);
}
