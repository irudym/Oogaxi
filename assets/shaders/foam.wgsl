#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// x - time, y - sparkle strength, z - foam thickness, w - top line width
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;


fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = params.x;
    let depth = mesh.uv.y;

    // create foam on the top
    if depth < params.w {
        let foam_band = 1.0 - smoothstep(0.0, params.z, depth);

        let froth = hash(vec2<f32>(floor(mesh.uv.x * 40.0), floor(t * 6.0)));
        let foam = foam_band * step(0.20, froth);

        return vec4<f32>(0.87, 0.95, 0.94, 1.0) + foam;
    }

    var out = vec3<f32>(0.0);

    // foam
    // smoothstep gives a soft lower edge

    //out += vec3<f32>(0.72, 0.88, 0.92) * 1.0  * 0.5;

    // sparkle
    let n1 = hash(floor(vec2<f32>(mesh.uv.x * 60.0 + t * 3.0, mesh.uv.y * 30.0)));
    let n2 = hash(floor(vec2<f32>(mesh.uv.x * 45.0 - t * 2.0, mesh.uv.y * 22.0)));

    let glint = pow(n1 * n2, 8.0) * params.y;
    let surface_bias = pow(1.0 - depth, 3.0); // glint only near the top
    out += vec3<f32>(1.0, 0.98, 0.9) * glint * surface_bias;

    return vec4<f32>(out, 1.0);
}
