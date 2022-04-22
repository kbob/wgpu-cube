// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct CameraUniform {
   view_proj: mat4x4<f32>;
};
[[group(0), binding(1)]]
var<uniform> camera: CameraUniform;

struct Light {
    color: vec4<f32>;
    direction: vec4<f32>;
    position: vec4<f32>;
    proj: mat4x4<f32>;
};
struct LightsUniform {
    count: u32;
    lights: array<Light, 8>;
};
[[group(0), binding(2)]]
var<uniform> lights: LightsUniform;

struct CubeUniform {
    cube_to_world: mat4x4<f32>;
    decal_visibility: f32;
};
[[group(1), binding(1)]]
var<uniform> cube: CubeUniform;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
};

fn extract3x3(in: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(in[0].xyz, in[1].xyz, in[2].xyz);
}

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    pos = cube.cube_to_world * pos;
    pos = camera.view_proj * pos;

    var normal = model.normal;
    let cube_to_world_normal = extract3x3(cube.cube_to_world);
    normal = cube_to_world_normal * normal;

    var out: VertexOutput;
    out.clip_position = pos;
    out.normal = normal;
    return out;
}

// Fragment shader

fn lambert_diffuse(normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    return max(0.0, dot(normal, light_dir));
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // return vec4<f32>(0.718, 0.055, 0.0, 1.0) ;// * 0.02;
    let normal = normalize(in.normal);
    let material_color = vec4<f32>(0.718, 0.055, 0.0, 1.0);
    var color = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = 0u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        var ambient = 0.0;
        var diffuse = 0.0;
        var light_has_direction = false;
        var light_dir = vec3<f32>(0.0);
        if (light.position.w == 0.0) {
            if (light.direction.w == 0.0) {
                // ambient light
                ambient = 1.0;
            } else {
                // directional light
                light_dir = light.direction.xyz;
                light_has_direction = true;
            }
        } else {
            light_dir = light.position.xyz - in.clip_position.xyz;
            if (light.direction.w == 0.0) {
                // point light
            } else {
                // spotlight
            }
        }

        if (light_has_direction) {
            light_dir = normalize(light_dir);
            diffuse = lambert_diffuse(normal, light_dir);
        }

        color = color + (ambient + diffuse) * light.color.rgb;
    }
    return vec4<f32>(color, 1.0) * material_color;
}
