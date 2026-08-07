#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::view
#import bevy_render::view::frag_coord_to_uv

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;
// x = time, y = sparkle, z = desaturation, w = refraction
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> params2: vec4<f32>;
// x = foam thickness, y = shimmer, z/w = spare
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var scene_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var scene_sampler: sampler;

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = params.x;
    let depth = mesh.uv.y;

    // sample the scene behid and refracted
    var screen_uv = frag_coord_to_uv(mesh.position.xy, view.viewport);
    let wobble = sin(screen_uv.x * 40.0 + t * 2.0) * (2.56/ view.viewport.z) + sin(screen_uv.y * 25.0 - t * 1.3) * (1.1 / view.viewport.w);
    screen_uv.x += wobble * params.w * (1.0 - depth);
    let scene = textureSample(scene_tex, scene_sampler, screen_uv).rgb;

    // desaturate
    let lum = dot(scene, vec3<f32>(0.299, 0.587, 0.114));
    let desat = mix(scene, vec3<f32>(lum*4.0), params.z);
    // let lum = (scene.r + scene.g + scene.b) / 3.0;
     // absorption
    let shallow = vec3<f32>(0.31, 0.70, 0.75);
    let deep = vec3<f32>(0.07, 0.23, 0.29);

    var out = desat * mix(shallow, deep, depth * depth );//vec3<f32>(1.0,1.0,1.0);

    if mesh.uv.y < params2.x {
        out = vec3<f32>(1.0, 1.0, 1.0);
    }

    /*
    let foam_band = 1.0 - smoothstep(0.0, params2.x, depth);
    let froth = hash(vec2<f32>(floor(mesh.uv.x * 90.0), floor(t * 6.0)));
    out += vec3<f32>(0.72, 0.88, 0.92) * foam_band * step(0.45, froth) * 0.5;
    */

    // --- ADD sparkle: rare sharp glints near the surface ---
    /*
    let n1 = hash(floor(vec2<f32>(mesh.uv.x * 60.0 + t * 3.0, mesh.uv.y * 30.0)));
    let n2 = hash(floor(vec2<f32>(mesh.uv.x * 45.0 - t * 2.0, mesh.uv.y * 22.0)));
    let glint = pow(n1 * n2, 8.0) * params.y * pow(1.0 - depth, 3.0);
    out += vec3<f32>(1.0, 0.98, 0.9) * glint;
    */

    return vec4<f32>(out, 1.0);
}
