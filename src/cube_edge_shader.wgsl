// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct CameraUniform {
   view_xform: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct CubeUniform {
    obj_xform: mat4x4<f32>;
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
    var out: VertexOutput;
    out.clip_position = camera.view_xform * vec4<f32>(model.position, 1.0);
    out.normal = vec3<f32>(1.0, 0.0, 0.0);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn fw_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.6, 0.4, 0.1, 1.0);
}