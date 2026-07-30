#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;

// params.x - vignette strenght
//       .y - scanline strenght
//       .z - time
//       .w - unused

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    // Vignette: darken towards the corners. Distance from center, squared for a soft falloff
    // that doesn't hide the playfield
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.35, 0.85, d) * params.x;

    let scan = 1.0; //1.0 - params.y * (0.5 + 0.5 * sin(uv.y * 800.0));

    let shade = vignette * scan;
    return vec4<f32>(0.0, 0.0, 0.0, 1.0 - shade);
}
