// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(1)]]
var<uniform> camera: CameraUniform;

struct CubeUniform {
    cube_to_world: mat4x4<f32>;
    decal_visibility: f32;
};
[[group(1), binding(1)]]
var<uniform> cube: CubeUniform;

struct InstanceStaticInput {
    [[location(5)]] face_to_cube_0: vec4<f32>;
    [[location(6)]] face_to_cube_1: vec4<f32>;
    [[location(7)]] face_to_cube_2: vec4<f32>;
    [[location(8)]] face_to_cube_3: vec4<f32>;
    [[location(9)]] decal_offset: vec2<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] decal_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0), interpolate(perspective, sample)]] normal: vec3<f32>;
    [[location(1), interpolate(perspective, sample)]] decal_coords: vec2<f32>;
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
    out.decal_coords = instance.decal_offset + model.decal_coords;
    return out;
}

// Fragment shader

[[group(1), binding(0)]]
var t_blinky: texture_2d<u32>;

[[group(0), binding(0)]]
var t_decal: texture_2d<f32>;

let face_base_color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
let led_base_color: vec4<f32> = vec4<f32>(0.04, 0.04, 0.04, 1.0);
let led_r2: f32 = 0.10;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // 0 <= x <= 6; 0 <= y <= 1
    let t_coord = vec2<f32>(in.decal_coords.x, 1.0 - in.decal_coords.y);
    let pix_coord = t_coord * 64.0;
    let pix_center = round(pix_coord);
    let tex_index = vec2<i32>(pix_center);

    let decal_color = vec4<f32>(textureLoad(t_decal, tex_index, 0));
    let blinky_color = vec4<f32>(textureLoad(t_blinky, tex_index, 0)) / 255.0;
    let face_color = max(face_base_color, cube.decal_visibility * decal_color);
    var led_color = max(led_base_color, blinky_color);

    let pix_pos = pix_coord - pix_center;
    let pix_r2: f32 = pix_pos.x * pix_pos.x + pix_pos.y * pix_pos.y;
    if (pix_r2 < led_r2) {
        return led_color;
    } else {
        return face_color;
    }
}
