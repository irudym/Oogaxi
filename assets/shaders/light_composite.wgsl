#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var light_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var light_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let light = textureSample(light_map, light_sampler, mesh.uv).rgb;
    // darken in inverse: alpha carries "how much shadow to apply"
    // color carries the light's tint
    let luminance = dot(light, vec3<f32>(0.7, 0.7, 0.7)); // (0.299, 0.587, 0.114)
    return vec4<f32>(light * 0.5, 1.0 - clamp(luminance, 0.0, 1.0));
}
