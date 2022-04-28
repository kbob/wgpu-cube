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
    shadow_map_size: f32;
    shadow_map_inv_size: f32;
};
struct LightsUniform {
    count: u32;
    lights: array<Light, 8>;
};
[[group(0), binding(2)]]
var<uniform> lights: LightsUniform;

struct ShadowUniform {
    proj: mat4x4<f32>;
};
[[group(2), binding(0)]]
var<uniform> shadow: ShadowUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] decal_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0), interpolate(perspective, sample)]] world_position: vec4<f32>;
    [[location(1), interpolate(perspective, sample)]] normal: vec3<f32>;
    [[location(2), interpolate(perspective, sample)]] decal_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let world_pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    let view_pos = camera.view_proj * world_pos;

    var normal: vec3<f32> = model.normal;

    var out: VertexOutput;
    out.clip_position = view_pos;
    out.world_position = world_pos;
    out.normal = normal;
    out.decal_coords = model.decal_coords;
    return out;
}

[[stage(vertex)]]
fn vs_shadow_main(
    model: VertexInput,
) -> [[builtin(position)]] vec4<f32> {
    let world_pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    let view_pos = shadow.proj * world_pos;
    return view_pos;
}

// Fragment shader

[[group(0), binding(3)]]
var t_floor_decal: texture_2d<f32>;
[[group(0), binding(4)]]
var s_floor_decal: sampler;

[[group(2), binding(0)]]
var t_shadow_maps: texture_depth_2d_array;
[[group(2), binding(1)]]
var s_shadow_maps: sampler_comparison;

// No base material color.  It comes from the decal texture.
let material_roughness: f32 = 0.6;

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

fn fetch_shadow(light_index: u32, homogeneous_coords: vec4<f32>) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and
    // texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);

    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction
        + vec2<f32>(0.5, 0.5);
    
    // skip sampling if texture index out of bounds
    let clamped = clamp(light_local, vec2<f32>(0.0), vec2<f32>(1.0));
    if (clamped.x != light_local.x || clamped.y != light_local.y) {
        return 1.0;
    }

    // do the lookup, using HW PCF and comparison
    return textureSampleCompareLevel(
        t_shadow_maps,
        s_shadow_maps,
        light_local,
        i32(light_index),
        homogeneous_coords.z * proj_correction
    );
}

// This uses four samples and and ordered dither to approximate a
// 16 sample average.
fn fetch_shadow4(light_index: u32, homogeneous_coords: vec4<f32>) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }

    let light = lights.lights[light_index];
    let map_size = light.shadow_map_size;
    let inv_map_size = light.shadow_map_inv_size;

    // compensate for the Y-flip difference between the NDC and
    // texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);

    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction
        + vec2<f32>(0.5, 0.5);

    // skip sampling if texture index out of bounds
    let clamped = clamp(light_local, vec2<f32>(0.0), vec2<f32>(1.0));
    if (clamped.x != light_local.x || clamped.y != light_local.y) {
        return 1.0;
    }

    // calculate dither matrix index
    var offset = vec2<f32>(
        f32(fract(map_size * light_local.x) > 0.5),
        f32(fract(map_size * light_local.y) > 0.5),
    );
    offset.y = offset.y + offset.x; // y ^= x in floating point
    if (offset.y > 1.1) {
        offset.y = 0.0;
    }

    // do the lookup.  Average four samples.  Each sample uses
    // HW PCF, comparison, and bias.
    let uv = light_local + (offset + 0.5) * inv_map_size;
    let s0 = textureSampleCompareLevel(
        t_shadow_maps,
        s_shadow_maps,
        uv,
        i32(light_index),
        homogeneous_coords.z * proj_correction
    );
    let s1 = textureSampleCompareLevel(
        t_shadow_maps,
        s_shadow_maps,
        uv,
        i32(light_index),
        homogeneous_coords.z * proj_correction,
        vec2<i32>(0, -2)
    );
    let s2 = textureSampleCompareLevel(
        t_shadow_maps,
        s_shadow_maps,
        uv,
        i32(light_index),
        homogeneous_coords.z * proj_correction,
        vec2<i32>(-2, 0)
    );
    let s3 = textureSampleCompareLevel(
        t_shadow_maps,
        s_shadow_maps,
        uv,
        i32(light_index),
        homogeneous_coords.z * proj_correction,
        vec2<i32>(-2, -2)
    );
    return 0.25 * (s0 + s1 + s2 + s3);
}

fn fetch_shadow16(light_index: u32, homogeneous_coords: vec4<f32>) -> f32 {

    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }

    let light = lights.lights[light_index];
    let inv_map_size = light.shadow_map_inv_size;

    // compensate for the Y-flip difference between the NDC and
    // texture coordinates.
    let flip_correction = vec2<f32>(0.5, -0.5);

    // compute texture coordinates for shadow lookup.
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction
        + vec2<f32>(0.5, 0.5);

    // skip sampling if texture index out of bounds.
    let clamped = clamp(light_local, vec2<f32>(0.0), vec2<f32>(1.0));
    if (clamped.x != light_local.x || clamped.y != light_local.y) {
        return 1.0;
    }

    // sample.  16 bilinear samples per fragment.
    let t = t_shadow_maps;
    let s = s_shadow_maps;
    let uv = light_local + 0.5 * inv_map_size;
    let i = i32(light_index);
    let z = 0.2;
    let s00 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-2, -2));
    let s01 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-2, -1));
    let s02 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-2, 0));
    let s03 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-2, 1));

    let s10 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-1, -2));
    let s11 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-1, -1));
    let s12 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-1, 0));
    let s13 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(-1, 1));

    let s20 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(0, -2));
    let s21 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(0, -1));
    let s22 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(0, 0));
    let s23 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(0, 1));

    let s30 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(1, -2));
    let s31 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(1, -1));
    let s32 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(1, 0));
    let s33 = textureSampleCompareLevel(t, s, uv, i, z, vec2<i32>(1, 1));

    return 0.0625 * (s00 + s01 + s02 + s03 +
                     s10 + s11 + s12 + s13 +
                     s20 + s21 + s22 + s23 +
                     s30 + s31 + s32 + s33);
}

fn floor_color(
    tex_coord: vec2<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    world_pos: vec4<f32>,
) -> vec4<f32> {
    let material_color =
        textureSample(t_floor_decal, s_floor_decal, tex_coord).rgb * 0.3;

    var color = vec3<f32>(0.0);

    // Ambient
    color = color + lights.lights[0].color.rgb * material_color;

    // Directional lights
    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let light_dir = normalize(light.direction.xyz);
        let half_dir = normalize(view_dir + light_dir);

        // let s1 = fetch_shadow(i, light.proj * world_pos);
        // let shadow = fetch_shadow4(i, light.proj * world_pos);
        let shadow = fetch_shadow16(i, light.proj * world_pos);
        // let shadow = 0.5 + s1 - s16;
    

        // let diffuse = lambert_diffuse(light.color.rgb, normal, light_dir);
        let diffuse = burley_diffuse(
            material_roughness,
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
        color = color + material_color * shadow * (diffuse + specular);
    }
    return vec4<f32>(color, 1.0);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let t_coord = vec2<f32>(in.decal_coords.x, 1.0 - in.decal_coords.y);

    let normal = normalize(in.normal);
    let view_dir = normalize(camera.view_position.xyz - in.world_position.xyz);

    return floor_color(t_coord, normal, view_dir, in.world_position);
}
