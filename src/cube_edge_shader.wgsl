// Vertex shader

struct CameraUniform {
    view_position: vec4<f32>;
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

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

fn extract3x3(in: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(in[0].xyz, in[1].xyz, in[2].xyz);
}

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    let world_pos = cube.cube_to_world * pos;
    pos = camera.view_proj * world_pos;

    var normal = model.normal;
    let cube_to_world_normal = extract3x3(cube.cube_to_world);
    normal = cube_to_world_normal * normal;

    var out: VertexOutput;
    out.clip_position = pos;
    out.world_position = world_pos.xyz;
    out.normal = normal;
    return out;
}

// Fragment shader

// let material_color = vec4<f32>(0.718, 0.055, 0.0, 1.0);
let material_color = vec4<f32>(0.05, 0.05, 0.05, 1.0);


fn lambert_diffuse(
    light_color: vec3<f32>,
    normal: vec3<f32>,
    light_dir: vec3<f32>
) -> vec3<f32> {
    return max(0.0, dot(normal, light_dir)) * light_color;
}

fn burley_diffuse(
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    )
-> f32 {
    return 0.0;
}

fn phong_specular(
    light_color: vec3<f32>,
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
) -> vec3<f32> {
    let reflect_dir = reflect(-light_dir, normal);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular_color = specular_strength * light_color;
    return specular_color;
}

fn blinn_phong_specular(
    light_color: vec3<f32>,
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
) -> vec3<f32> {
    let half_dir = normalize(view_dir + light_dir);
    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light_color;
    return specular_color;
}

fn edge_color(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
) -> vec4<f32> {
    var color = vec3<f32>(0.0);

    // Ambient
    color = color + lights.lights[0].color.rgb * material_color.rgb;

    // Directional lights
    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        var light_dir = normalize(light.direction.xyz);

        let diffuse = lambert_diffuse(light.color.rgb, normal, light_dir);
        let specular =
            blinn_phong_specular(light.color.rgb, normal, light_dir, view_dir);
        color = color + material_color.rgb * (diffuse + specular);
    }
    return vec4<f32>(color, material_color.a);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let normal = normalize(in.normal);
    let view_dir = normalize(camera.view_position.xyz - in.world_position);

    let color = edge_color(normal, view_dir);
    return color;
    
    // var color = vec3<f32>(0.0, 0.0, 0.0);
    // for (var i = 0u; i < lights.count; i = i + 1u) {
    //     let light = lights.lights[i];
    //     var ambient = 0.0;
    //     var diffuse = 0.0;
    //     var light_has_direction = false;
    //     var light_dir = vec3<f32>(0.0);
    //     if (light.position.w == 0.0) {
    //         if (light.direction.w == 0.0) {
    //             // ambient light
    //             ambient = 1.0;
    //         } else {
    //             // directional light
    //             light_dir = light.direction.xyz;
    //             light_has_direction = true;
    //         }
    //     } else {
    //         light_dir = light.position.xyz - in.clip_position.xyz;
    //         if (light.direction.w == 0.0) {
    //             // point light
    //         } else {
    //             // spotlight
    //         }
    //     }

    //     if (light_has_direction) {
    //         light_dir = normalize(light_dir);
    //         diffuse = lambert_diffuse(normal, light_dir);
    //     }

    //     color = color + (ambient + diffuse) * light.color.rgb;
    // }
    // return vec4<f32>(color, 1.0) * material_color;
}
