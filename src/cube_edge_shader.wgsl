// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct CameraUniform {
   view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct CubeUniform {
    cube_to_world: mat4x4<f32>;
    decal_is_visible: u32;
};
[[group(2), binding(0)]]
var<uniform> cube: CubeUniform;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    pos = cube.cube_to_world * pos;
    pos = camera.view_proj * pos;

    var out: VertexOutput;
    out.clip_position = pos;
    out.normal = vec3<f32>(1.0, 0.0, 0.0);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.718, 0.055, 0.0, 1.0) ;// * 0.02;
}
