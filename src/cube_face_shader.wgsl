// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    pos = camera.view_proj * pos;
    var out: VertexOutput;
    out.clip_position = pos;
    out.normal = model.normal;
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader

[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let t_coord = vec2<f32>(in.tex_coords[0], 1.0 - in.tex_coords[1]);
    let tex = textureSample(t_diffuse, s_diffuse, t_coord);
    let c = vec3<f32>(0.3, 0.7, 0.6);
    let r = c[0] + 0.5 * in.tex_coords[0] - tex[0];
    let g = c[1] + 0.5 * in.tex_coords[1] - tex[1];
    let b = c[2] - tex[2];
    let a = tex[3];
    return vec4<f32>(r, g, b, a);

    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // let r = 0. + 0.5 * in.tex_coords[0];
    // let g = 0. + 0.5 * in.tex_coords[1];
    // let b = 0.;
    // let a = 1.0;
    // return vec4<f32>(r, g, b, a);

    // return vec4<f32>(0.6, 0.2, 0.3, 1.0);
}
