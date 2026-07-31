#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;
// x = time, y = shimmer strength, z = tint mix, w = unused
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var noise_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = params.x;

    // Two noise samples scrolling at different speeds and directions.
    // ONE layer looks like a moving texture; TWO look like light on water,
    // because neither speed dominates and the interference reads as organic.
    let n1 = textureSample(noise_tex, noise_sampler, mesh.uv * vec2(3.0, 1.0) + vec2(t * 0.05, 0.0)).r;
    let n2 = textureSample(noise_tex, noise_sampler, mesh.uv * vec2(2.0, 1.5) - vec2(t * 0.03, t * 0.01)).r;
    let shimmer = pow(n1 * n2, 3.0) * params.y;   // pow sharpens blobs into glints

    // Caustics concentrate near the surface — fade them with depth (uv.y).
    let surface_bias = 1.0 - mesh.uv.y;
    let glint = shimmer * surface_bias * surface_bias;

    let base = mesh.color;                         // depth gradient from vertices
    return vec4<f32>(base.rgb + vec3(glint), base.a);
}
