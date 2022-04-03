// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct CubeUniform {
    cube_to_world: mat4x4<f32>;
    decal_is_visible: u32;
};
[[group(1), binding(0)]]
var<uniform> cube: CubeUniform;

struct InstanceStaticInput {
    [[location(5)]] face_to_cube_0: vec4<f32>;
    [[location(6)]] face_to_cube_1: vec4<f32>;
    [[location(7)]] face_to_cube_2: vec4<f32>;
    [[location(8)]] face_to_cube_3: vec4<f32>;
    [[location(9)]] tex_offset: vec2<f32>;
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
    instance: InstanceStaticInput,
) -> VertexOutput {
    let face_to_cube = mat4x4<f32>(
        instance.face_to_cube_0,
        instance.face_to_cube_1,
        instance.face_to_cube_2,
        instance.face_to_cube_3,
    );

    var pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    pos = face_to_cube * pos;
    pos = cube.cube_to_world * pos;
    pos = camera.view_proj * pos;

    var out: VertexOutput;
    out.clip_position = pos;
    out.normal = model.normal;
    out.tex_coords = instance.tex_offset + model.tex_coords;
    return out;
}

// Fragment shader

[[group(2), binding(0)]]
var t_decal: texture_2d<f32>;
[[group(2), binding(1)]]
var s_decal: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // 0 <= x <= 6; 0 <= y <= 1
    let t_coord = vec2<f32>(in.tex_coords.x, 1.0 - in.tex_coords.y);
    let pixel_scale: f32 = 64.0;
    let pix_coord = t_coord * 64.0;
    let pix_center = round(pix_coord);
    let tex_index = pix_center / vec2<f32>(64.0 * 6.0, 64.0);
    var face_color = vec4<f32>(0.020, 0.020, 0.025, 1.0);
    if (cube.decal_is_visible != 0u) {
        let decal_color = textureSample(t_decal, s_decal, tex_index);
        face_color = max(face_color, decal_color);
    }
    let pix_pos = pix_coord - pix_center;
    let r2: f32 = pix_pos.x * pix_pos.x + pix_pos.y * pix_pos.y;
    if (r2 < 0.10) {
        return 0.0 * vec4<f32>(0.5, 0.0, 0.3, 1.0);
    }
    return face_color;
}
