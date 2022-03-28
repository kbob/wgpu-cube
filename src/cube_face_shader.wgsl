// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct InstanceInput {
    [[location(5)]] cube_to_world_0: vec4<f32>;
    [[location(6)]] cube_to_world_1: vec4<f32>;
    [[location(7)]] cube_to_world_2: vec4<f32>;
    [[location(8)]] cube_to_world_3: vec4<f32>;

    [[location(9)]] face_to_cube_0: vec4<f32>;
    [[location(10)]] face_to_cube_1: vec4<f32>;
    [[location(11)]] face_to_cube_2: vec4<f32>;
    [[location(12)]] face_to_cube_3: vec4<f32>;
};

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
    instance: InstanceInput,
) -> VertexOutput {
    let cube_to_world = mat4x4<f32>(
        instance.cube_to_world_0,
        instance.cube_to_world_1,
        instance.cube_to_world_2,
        instance.cube_to_world_3,
    );
    let face_to_cube = mat4x4<f32>(
        instance.face_to_cube_0,
        instance.face_to_cube_1,
        instance.face_to_cube_2,
        instance.face_to_cube_3,
    );

    var pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    pos = face_to_cube * pos;
    pos = cube_to_world * pos;
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
    let tex = textureSample(t_diffuse, s_diffuse, t_coord) * 3.0 - 1.0;
    if (tex[2] < 0.1) {
        discard;
    }
    // let c = vec3<f32>(0.3, 0.7, 0.6);
    // let r = c[0] + 0.5 * in.tex_coords[0] - tex[0];
    // let g = c[1] + 0.5 * in.tex_coords[1] - tex[1];
    // let b = c[2] - tex[2];
    // let a = tex[3];
    let r = min(in.tex_coords[0] * in.tex_coords[0], tex[2]);
    let g = min(in.tex_coords[1] * in.tex_coords[1], tex[2]);
    let b = 0.0;
    let a = tex[2];
    return vec4<f32>(r, g, b, a);

    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // let r = 0. + 0.5 * in.tex_coords[0];
    // let g = 0. + 0.5 * in.tex_coords[1];
    // let b = 0.;
    // let a = 1.0;
    // return vec4<f32>(r, g, b, a);

    // return vec4<f32>(0.6, 0.2, 0.3, 1.0);
}
