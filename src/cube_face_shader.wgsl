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

struct ShadowUniform {
    proj: mat4x4<f32>;
};
[[group(2), binding(0)]]
var<uniform> shadow: ShadowUniform;

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
    [[location(0), interpolate(perspective, sample)]] world_position: vec3<f32>;
    [[location(1), interpolate(perspective, sample)]] normal: vec3<f32>;
    [[location(2), interpolate(perspective, sample)]] decal_coords: vec2<f32>;
};

fn extract3x3(in: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(in[0].xyz, in[1].xyz, in[2].xyz);
}

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

    let model_pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    let cube_pos: vec4<f32> = face_to_cube * model_pos;
    let world_pos = cube.cube_to_world * cube_pos;
    let view_pos = camera.view_proj * world_pos;

    var normal: vec3<f32> = model.normal;
    let face_to_cube_normal = extract3x3(face_to_cube);
    let cube_to_world_normal = extract3x3(cube.cube_to_world);
    normal = cube_to_world_normal * face_to_cube_normal * normal;

    var out: VertexOutput;
    out.clip_position = view_pos;
    out.world_position = world_pos.xyz;
    out.normal = normal;
    out.decal_coords = instance.decal_offset + model.decal_coords;
    return out;
}

[[stage(vertex)]]
fn vs_shadow_main(
    model: VertexInput,
    instance: InstanceStaticInput,
) -> [[builtin(position)]] vec4<f32> {
    let face_to_cube = mat4x4<f32>(
        instance.face_to_cube_0,
        instance.face_to_cube_1,
        instance.face_to_cube_2,
        instance.face_to_cube_3,
    );

    let model_pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    let cube_pos: vec4<f32> = face_to_cube * model_pos;
    let world_pos = cube.cube_to_world * cube_pos;
    let proj_pos = shadow.proj * world_pos;
    return proj_pos;
}

// Fragment shader

[[group(1), binding(0)]]
var t_blinky: texture_2d<u32>;

[[group(0), binding(0)]]
var t_decal: texture_2d<f32>;

let face_material_color: vec4<f32> = vec4<f32>(0.02, 0.02, 0.02, 1.0);
let face_material_roughness: f32 = 0.6;
let face_base_color: vec4<f32> = vec4<f32>(0.02, 0.02, 0.02, 1.0);
// 0.06 is more realistic.  0.0 has higher contrast.
// let led_base_color: vec4<f32> = vec4<f32>(0.06, 0.06, 0.06, 1.0);
let led_base_color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
let led_r2: f32 = 0.15;
let led_bleed_r2: f32 = 0.20;

fn lambert_diffuse(
    light_color: vec3<f32>,
    normal: vec3<f32>,
    light_dir: vec3<f32>
) -> vec3<f32> {
    return max(0.0, dot(normal, light_dir)) * light_color;
}

fn fifth(x: f32) -> f32 {
    let square = x * x;
    return square + square * x;
}

// fd = (baseColor / pi)
//     * (1 + (FD90 - 1) * (1 - cos(θl))**5)
//     * (1 + (FD90 - 1) * (1 - cos(θv))**5)
// FD90 = 0.5 + 2 * roughness * cos(θd)**2
fn burley_diffuse(
    material_roughness: f32,
    normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    half_dir: vec3<f32>,
    )
-> f32 {
    let cos_theta_l = dot(light_dir, normal);
    let cos_theta_v = dot(view_dir, normal);
    let cos_theta_d = dot(light_dir, half_dir);
    let fd90 = 0.5 + 2.0 * material_roughness * cos_theta_d * cos_theta_d;
    let f1 = 1.0 / 3.1415927;
    let f2 = 1.0 + (fd90 - 1.0) * fifth(1.0 - cos_theta_l);
    let f3 = 1.0 + (fd90 - 1.0) * fifth(1.0 - cos_theta_v);
    return f1 * f2 * f3;
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
    half_dir: vec3<f32>,
) -> vec3<f32> {
    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light_color;
    return specular_color;
}

fn face_color(
    tex_index: vec2<i32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
) -> vec4<f32> {
    var decal_pixel = vec4<f32>(textureLoad(t_decal, tex_index, 0));
    decal_pixel = cube.decal_visibility * decal_pixel;
    var material_color = face_base_color.rgb;
    material_color = max(material_color, 2.0 * decal_pixel.rgb);
    var color = vec3<f32>(0.0);

    // Ambient
    color = color + lights.lights[0].color.rgb * material_color;

    // Directional lights
    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let light_dir = normalize(light.direction.xyz);
        let half_dir = normalize(view_dir + light_dir);

        // let diffuse = lambert_diffuse(light.color.rgb, normal, light_dir);
        let diffuse = burley_diffuse(
            face_material_roughness,
            normal,
            light_dir,
            view_dir,
            half_dir,
        );
        let specular = blinn_phong_specular(
            light.color.rgb,
            normal,
            light_dir,
            view_dir,
            half_dir,
        );
        color = color + material_color * (diffuse + specular);
    }
    let alpha = face_base_color.a * decal_pixel.a;
    return vec4<f32>(color, alpha);
}

fn led_color(
    tex_index: vec2<i32>,
) -> vec4<f32> {
    let blinky_color = vec4<f32>(textureLoad(t_blinky, tex_index, 0)) / 255.0;
    let led_color = max(led_base_color, blinky_color);
    return led_color;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let t_coord = vec2<f32>(in.decal_coords.x, 1.0 - in.decal_coords.y);
    let pix_coord = t_coord * 64.0;
    let pix_center = floor(pix_coord) + 0.5;
    let tex_index = vec2<i32>(pix_center);

    let normal = normalize(in.normal);
    let view_dir = normalize(camera.view_position.xyz - in.world_position);

    let face_color = face_color(tex_index, normal, view_dir);
    let led_color = led_color(tex_index);

    let pix_pos = pix_coord - pix_center;
    let pix_r2: f32 = pix_pos.x * pix_pos.x + pix_pos.y * pix_pos.y;
    if (pix_r2 < led_r2) {
        return led_color;
    } else {
        return face_color;
    }
}
