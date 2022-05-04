// All shaders in one file to eliminate duplicate code.

// ----  Common Data   ---- ---- ---- ---- ---- ---- ---- ---- ---- ----

struct CameraUniform {
    view_position: vec4<f32>;
    world_to_clip: mat4x4<f32>;
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

struct CubeUniform {
    cube_to_world: mat4x4<f32>;
    decal_visibility: f32;
};
[[group(1), binding(1)]]
var<uniform> cube: CubeUniform;

struct ShadowUniform {
    world_to_clip: mat4x4<f32>;
};
[[group(2), binding(0)]]
var<uniform> shadow: ShadowUniform;

let TAU: f32 = 6.283185307179586;
let PI: f32 = 3.141592653589793;

let USE_BRDF_FLAG: bool = true;

// ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ====
// ====  Vertex Shaders


// ----  Common Vertex Shader Functions  - ---- ---- ---- ---- ---- ----

fn extract3x3(in: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(in.x.xyz, in.y.xyz, in.z.xyz);
}


// ----  Cube Face Vertex Shader  --- ---- ---- ---- ---- ---- ---- ----

struct CubeFaceInstanceInput {
    [[location(5)]] face_to_cube_0: vec4<f32>;
    [[location(6)]] face_to_cube_1: vec4<f32>;
    [[location(7)]] face_to_cube_2: vec4<f32>;
    [[location(8)]] face_to_cube_3: vec4<f32>;
    [[location(9)]] decal_offset: vec2<f32>;
};

struct CubeFaceVertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] decal_coords: vec2<f32>;
};

struct CubeFaceVertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0), interpolate(perspective, sample)]] world_position: vec4<f32>;
    [[location(1), interpolate(perspective, sample)]] world_normal: vec3<f32>;
    [[location(2), interpolate(perspective, sample)]] decal_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_cube_face_main(
    model: CubeFaceVertexInput,
    instance: CubeFaceInstanceInput,
) -> CubeFaceVertexOutput {
    let face_to_cube = mat4x4<f32>(
        instance.face_to_cube_0,
        instance.face_to_cube_1,
        instance.face_to_cube_2,
        instance.face_to_cube_3,
    );

    let model_pos = vec4<f32>(model.position, 1.0);
    let cube_pos = face_to_cube * model_pos;
    let world_pos = cube.cube_to_world * cube_pos;
    let clip_pos = camera.world_to_clip * world_pos;

    let face_normal = model.normal;
    let cube_normal = extract3x3(face_to_cube) * face_normal;
    let world_normal = extract3x3(cube.cube_to_world) * cube_normal;

    let decal_coords = instance.decal_offset + model.decal_coords;

    var out: CubeFaceVertexOutput;
    out.clip_position = clip_pos;
    out.world_position = world_pos;
    out.world_normal = world_normal;
    out.decal_coords = decal_coords;
    return out;
}

// face vertex shader for shadow pass
[[stage(vertex)]]
fn vs_cube_face_shadow_main(
    model: CubeFaceVertexInput,
    instance: CubeFaceInstanceInput,
) -> [[builtin(position)]] vec4<f32> {
    let face_to_cube = mat4x4<f32>(
        instance.face_to_cube_0,
        instance.face_to_cube_1,
        instance.face_to_cube_2,
        instance.face_to_cube_3,
    );

    let model_pos = vec4<f32>(model.position, 1.0);
    let cube_pos = face_to_cube * model_pos;
    let world_pos = cube.cube_to_world * cube_pos;
    let clip_pos = shadow.world_to_clip * world_pos;

    return clip_pos;
}


// ----  Cube Face Vertex Shader  --- ---- ---- ---- ---- ---- ---- ----

struct CubeEdgeVertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct CubeEdgeVertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
};

[[stage(vertex)]]
fn vs_cube_edge_main(
    model: CubeEdgeVertexInput,
) -> CubeEdgeVertexOutput {
    let cube_pos = vec4<f32>(model.position, 1.0);
    let world_pos = cube.cube_to_world * cube_pos;
    let clip_pos = camera.world_to_clip * world_pos;

    let cube_normal = model.normal;
    let world_normal = extract3x3(cube.cube_to_world) * cube_normal;

    var out: CubeEdgeVertexOutput;
    out.clip_position = clip_pos;
    out.world_position = world_pos;
    out.world_normal = world_normal;
    return out;
}

// edge vertex shader for shadow pass
[[stage(vertex)]]
fn vs_cube_edge_shadow_main(
    model: CubeEdgeVertexInput,
) -> [[builtin(position)]] vec4<f32> {
    let cube_pos = vec4<f32>(model.position, 1.0);
    let world_pos = cube.cube_to_world * cube_pos;
    let clip_pos = shadow.world_to_clip * world_pos;
    return clip_pos;
}

// ----  Floor Vertex Shader  -- ---- ---- ---- ---- ---- ---- ---- ----

struct FloorVertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] decal_coords: vec2<f32>;
};

struct FloorVertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0), interpolate(perspective, sample)]] world_position: vec4<f32>;
    [[location(1), interpolate(perspective, sample)]] world_normal: vec3<f32>;
    [[location(2), interpolate(perspective, sample)]] decal_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_floor_main(
    model: FloorVertexInput,
) -> FloorVertexOutput {
    let world_pos: vec4<f32> = vec4<f32>(model.position, 1.0);
    let clip_pos = camera.world_to_clip * world_pos;

    let world_normal: vec3<f32> = model.normal;

    var out: FloorVertexOutput;
    out.clip_position = clip_pos;
    out.world_position = world_pos;
    out.world_normal = world_normal;
    out.decal_coords = model.decal_coords;
    return out;
}

[[stage(vertex)]]
fn vs_floor_shadow_main(
    model: FloorVertexInput,
) -> [[builtin(position)]] vec4<f32> {
    let world_pos = vec4<f32>(model.position, 1.0);
    let clip_pos = shadow.world_to_clip * world_pos;
    return clip_pos;
}


// ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ==== ====
// ====  Fragment Shaders

[[group(2), binding(0)]]
var t_shadow_maps: texture_depth_2d_array;
[[group(2), binding(1)]]
var s_shadow_maps: sampler_comparison;


// ----  "Disney" BRDF  --- ---- ---- ---- ---- ---- ---- ---- ---- ----

struct Material {
    base_color: vec3<f32>;
    subsurface: f32;
    metallic: f32;
    specular: f32;
    specular_tint: f32;
    roughness: f32;
    anisotropic: f32;
    sheen: f32;
    sheen_tint: f32;
    clearcoat: f32;
    clearcoat_gloss: f32;
};

fn material_defaults() -> Material {
    var out: Material;
    out.base_color = vec3<f32>(0.82, 0.67, 0.16);
    out.subsurface = 0.0;
    out.metallic = 0.0;
    out.specular = 0.5;
    out.specular_tint = 0.0;
    out.roughness = 0.5;
    out.anisotropic = 0.0;
    out.sheen = 0.5;
    out.sheen_tint = 0.5;
    out.clearcoat = 0.0;
    out.clearcoat_gloss = 1.0;
    return out;
}

fn sqr(x: f32) -> f32 {
    return x * x;
}

fn schlick_fresnel(u: f32) -> f32 {
    let m = clamp(1.0 - u, 0.0, 1.0);
    let m2 = m * m;
    return m2 * m2 * m; // pow(m, 5)
}

// generalized Trowbridge-Reitz distribution, gamma=1
fn gtr1(NdotH: f32, a: f32) -> f32 {
    if (a >= 1.0) {
        return 1.0 / PI;
    }
    let a2 = a * a;
    let t = 1.0 + (a2 - 1.0) * NdotH * NdotH;
    return (a2 - 1.0) / (PI * log(a2) * t);
}

// generalized Trowbridge-Reitz distribution, gamma=2
fn gtr2(NdotH: f32, a: f32) -> f32 {
    let a2 = a * a;
    let t = 1.0 + (a2 - 1.0) * NdotH * NdotH;
    return a2 / (PI * t * t);
}

fn gtr2_aniso(NdotH: f32, HdotX: f32, HdotY: f32, ax: f32, ay: f32) -> f32 {
    return 1.0
        / (PI
            * ax
            * ay
            * sqr(sqr(HdotX / ax) + sqr(HdotY / ay) + NdotH * NdotH));
}

fn smithg_ggx(NdotV: f32, alphaG: f32) -> f32 {
    let a = alphaG * alphaG;
    let b = NdotV * NdotV;
    return 1.0 / (NdotV + sqrt(a + b +- a * b));
}

fn smithg_ggx_aniso(
    NdotV: f32,
    VdotX: f32,
    VdotY: f32,
    ax: f32,
    ay: f32,
) -> f32 {
    return 1.0
        / (NdotV +
            sqrt(sqr(VdotX * ax) + sqr(VdotY * ay) + sqr(NdotV)));
}

fn mon2lin(x: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(pow(x[0], 2.2), pow(x[1], 2.2), pow(x[2], 2.2));
}

fn disney_brdf(
    material: Material,
    L: vec3<f32>,
    V: vec3<f32>,
    N: vec3<f32>,
    X: vec3<f32>,
    Y: vec3<f32>,
) -> vec3<f32> {
    let NdotL = dot(N, L);
    let NdotV = dot(N, V);
    if (NdotL < 0.0 || NdotV < 0.0) {
        return vec3<f32>(0.0);
    }

    let H = normalize(L + V);
    let NdotH = dot(N, H);
    let LdotH = dot(L, H);

    let Cdlin = mon2lin(material.base_color);
    // luminance approx.
    let Cdlum = 0.3 * Cdlin[0] + 0.6 * Cdlin[1] + 0.1 * Cdlin[2];

    var Ctint: vec3<f32> = vec3<f32>(1.0);
    if (Cdlum > 0.0) {
        Ctint = Cdlin / Cdlum;
    }
    let Cspec0 = mix(
        material.specular * 0.08 * mix(
            vec3<f32>(1.0),
            Ctint,
            material.specular_tint,
        ),
        Cdlin,
        material.metallic,
    );
    let Csheen = mix(vec3<f32>(1.0), Ctint, material.sheen_tint);

    // Diffuse fresnel - go from 1 at normal incidence to .5 at grazing
    // and mix in diffuse retro-reflection based on roughness.
    let FL = schlick_fresnel(NdotL);
    let FV = schlick_fresnel(NdotV);
    let Fd90 = 0.5 + 2.0 * LdotH * LdotH * material.roughness;
    let Fd = mix(1.0, Fd90, FL) * mix(1.0, Fd90, FV);

    // Based on Hanrahan-Krueger brdf approximation of isotropic bssrdf
    // 1.25 scale is used to (roughly) preserve albedo
    // Fss90 used to "flatten" retroreflection based on roughness
    let Fss90 = LdotH * LdotH * material.roughness;
    let Fss = mix(1.0, Fss90, FL) * mix(1.0, Fss90, FV);
    let ss = 1.25 * (Fss * (1.0 / (NdotL + NdotV) - 0.5) + 0.5);

    // specular
    let aspect = sqrt(1.0 - material.anisotropic * 0.9);
    let ax = max(0.001, sqr(material.roughness) / aspect);
    let ay = max(0.001, sqr(material.roughness) * aspect);
    let Ds = gtr2_aniso(NdotH, dot(H, X), dot(H, Y), ax, ay);

    let FH = schlick_fresnel(LdotH);
    let Fs = mix(Cspec0, vec3<f32>(1.0), FH);
    let Gs = smithg_ggx_aniso(NdotL, dot(L, X), dot(L, Y), ax, ay)
        * smithg_ggx_aniso(NdotV, dot(V, X), dot(V, Y), ax, ay);

    // sheen
    let Fsheen = FH * material.sheen * Csheen;

    // clearcoat (index of refraction = 1.5 -> F0 = 0.04)
    let Dr = gtr1(NdotH, mix(0.1, 0.001, material.clearcoat_gloss));
    let Fr = mix(0.04, 1.0, FH);
    let Gr = smithg_ggx(NdotL, 0.25) * smithg_ggx(NdotV, 0.25);

    return ((1.0 / PI) * mix(Fd, ss, material.subsurface) * Cdlin + Fsheen)
        * (1.0 - material.metallic)
        + Gs * Fs * Ds
        + 0.25 * material.clearcoat * Gr * Fr * Dr;
}


// ----  Common Fragment Shader Functions  ---- ---- ---- ---- ---- ----

fn lambert_diffuse(
    light_color: vec3<f32>,
    normal: vec3<f32>,
    light_dir: vec3<f32>
) -> vec3<f32> {
    return max(0.0, dot(normal, light_dir)) * light_color;
}

fn fifth_power(x: f32) -> f32 {
    let square = x * x;
    return square + square * x;
}

// Burley diffuse shading
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
    if (cos_theta_l < 0.0 || cos_theta_v < 0.0) {
        return 0.0;
    }
    let fd90 = 0.5 + 2.0 * material_roughness * cos_theta_d * cos_theta_d;
    let f1 = 1.0 / 3.1415927;
    let f2 = 1.0 + (fd90 - 1.0) * fifth_power(1.0 - cos_theta_l);
    let f3 = 1.0 + (fd90 - 1.0) * fifth_power(1.0 - cos_theta_v);
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

// Single sample shadow.  Sharp edged and jaggy
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

// Nice looking, expensive soft shadow w/ 16 shadow map samples
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


// ----  Cube Face Fragment Shader  - ---- ---- ---- ---- ---- ---- ----

[[group(0), binding(0)]]
var t_decal: texture_2d<f32>;

[[group(1), binding(0)]]
var t_blinky: texture_2d<u32>;

let cube_face_material_color: vec4<f32> = vec4<f32>(0.02, 0.02, 0.02, 1.0);
let cube_face_material_roughness: f32 = 0.6;
let cube_face_base_color: vec4<f32> = vec4<f32>(0.02, 0.02, 0.02, 1.0);
// 0.06 is more realistic.  0.0 has higher contrast.
// let led_base_color: vec4<f32> = vec4<f32>(0.06, 0.06, 0.06, 1.0);
let led_base_color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
let led_r2: f32 = 0.15;
let led_brightness: f32 = 4.0;

fn face_color_brdf(
    tex_index: vec2<i32>,
    N: vec3<f32>,
    V: vec3<f32>,
    world_pos: vec4<f32>,
) -> vec4<f32> {
    let X = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), N));
    let Y = normalize(cross(N, X));

    let decal_pixel = vec4<f32>(textureLoad(t_decal, tex_index, 0));

    var material = material_defaults();
    material.base_color = vec3<f32>(0.2);
    material.specular = 0.15;
    material.roughness = 0.4;
    if (dot(decal_pixel, decal_pixel) != 1.0) {
        // this fragment is in a decal.
        material.base_color = decal_pixel.rgb;
        material.metallic = 0.8;
        material.specular = 0.4;
        material.roughness = 0.15;
        material.specular_tint = 1.0;
    }

    var color = vec3<f32>(0.0);

    // Ambient (cheating)
    color = color + lights.lights[0].color.rgb * material.base_color;

    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let L = normalize(light.direction.xyz);

        let shadow: f32 = 1.0;

        let b = max(vec3<f32>(0.0), disney_brdf(material, L, V, N, X, Y));
        color = color + dot(L, N) * light.color.rgb * b;

    }
    return vec4<f32>(color, 1.0);
}

fn face_color_classic(
    tex_index: vec2<i32>,
    N: vec3<f32>,
    V: vec3<f32>,
    world_pos: vec4<f32>,
) -> vec4<f32> {
    let decal_pixel = vec4<f32>(textureLoad(t_decal, tex_index, 0));
    var material_color = cube_face_base_color.rgb;
    material_color = max(material_color, decal_pixel.rgb);
    var color = vec3<f32>(0.0);

    // Ambient
    color = color + lights.lights[0].color.rgb * material_color;

    // Directional lights
    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let L = normalize(light.direction.xyz);
        let H = normalize(V + L);

        // shadow just adds shadow acne artifacts.  Skip it.
        // let shadow = fetch_shadow(i, light.proj * world_pos);
        let shadow = 1.0;

        let diffuse = lambert_diffuse(light.color.rgb, N, L);
        // let diffuse = burley_diffuse(
        //     cube_face_material_roughness,
        //     N,
        //     L,
        //     V,
        //     H,
        // );
        let specular = blinn_phong_specular(
            light.color.rgb,
            N,
            L,
            V,
            H,
        );
        color = color + shadow * material_color * (diffuse + specular);
    }
    let alpha = cube_face_base_color.a * decal_pixel.a;
    return vec4<f32>(color, alpha);
}

fn led_color(
    tex_index: vec2<i32>,
) -> vec4<f32> {
    let blinky_color = vec4<f32>(textureLoad(t_blinky, tex_index, 0)) / 255.0;
    let led_color = max(led_base_color, led_brightness * blinky_color);
    return led_color;
}

struct CubeFaceFragmentOutput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] bright_color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_cube_face_main(in: CubeFaceVertexOutput) -> CubeFaceFragmentOutput {
    let t_coord = vec2<f32>(in.decal_coords.x, 1.0 - in.decal_coords.y);
    let pix_coord = t_coord * 64.0;
    let pix_center = floor(pix_coord) + 0.5;
    let tex_index = vec2<i32>(pix_center);

    let world_pos = in.world_position;
    let N = normalize(in.world_normal);
    let V = normalize(camera.view_position.xyz - world_pos.xyz);

    let pix_pos = pix_coord - pix_center;
    let pix_r2: f32 = pix_pos.x * pix_pos.x + pix_pos.y * pix_pos.y;
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (pix_r2 < led_r2) {
        color = led_color(tex_index);
    } else if (USE_BRDF_FLAG) {
        color = face_color_brdf(tex_index, N, V, world_pos);
    } else {
        color = face_color_classic(tex_index, N, V, world_pos);
    }
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    var bright_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (brightness > 1.0) {
        bright_color = vec4<f32>(color.rgb, 1.0);
    }
    var out: CubeFaceFragmentOutput;
    out.color = color;
    out.bright_color = bright_color;
    return out;
}


// ----  Cube Edge Fragment Shader  - ---- ---- ---- ---- ---- ---- ----

// let cube_edge_material_color = vec4<f32>(0.718, 0.055, 0.0, 1.0);
// let cube_edge_material_color = vec4<f32>(0.0, 0.99, 1.0, 1.0);
let cube_edge_material_color = vec4<f32>(0.05, 0.05, 0.05, 1.0);
let cube_edge_material_roughness = 0.1;

fn edge_color_brdf(
    N: vec3<f32>,
    V: vec3<f32>,
    world_pos: vec4<f32>,
) -> vec4<f32> {
    let X = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), N));
    let Y = normalize(cross(N, X));

    var material = material_defaults();
    material.base_color = vec3<f32>(0.0);
    material.roughness = 0.05;

    var color = vec3<f32>(0.0);

    // Ambient (cheating)
    color = color + lights.lights[0].color.rgb * material.base_color;

    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let L = normalize(light.direction.xyz);

        let shadow: f32 = 1.0;

        let b = max(vec3<f32>(0.0), disney_brdf(material, L, V, N, X, Y));
        color = color + dot(L, N) * light.color.rgb * b;

    }
    return vec4<f32>(color, 1.0);
}

fn edge_color_classic(
    N: vec3<f32>,
    V: vec3<f32>,
    world_pos: vec4<f32>,
) -> vec4<f32> {
    var color = vec3<f32>(0.0);
    var material_color = cube_edge_material_color.rgb;

    // Ambient
    color = color + lights.lights[0].color.rgb * material_color;

    // Directional lights
    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let L = normalize(light.direction.xyz);
        let H = normalize(V + L);

        // shadow just adds shadow acne artifacts.  Skip it.
        // let shadow = fetch_shadow(i, light.proj * world_pos);
        let shadow = 1.0;

        let diffuse = lambert_diffuse(light.color.rgb, N, L);
        // let rough = cube_edge_material_roughness;
        // let diffuse = burley_diffuse(rough, N, L, V, H);

        let specular = blinn_phong_specular(light.color.rgb, N, L, V, H);

        color = color + shadow * material_color * (diffuse + specular);
    }
    return vec4<f32>(color, cube_edge_material_color.a);
}

// [[stage(fragment)]]
// fn YYYfs_cube_edge_main(in: CubeEdgeVertexOutput) -> [[location(0)]] vec4<f32> {
//     let N = normalize(in.world_normal);
//     let X = normalize(cross(N, vec3<f32>(1.0, 0.0, 0.0))); // arbitrary
//     let Y = normalize(cross(N, X));
//     let V = normalize(camera.view_position.xyz - in.world_position.xyz);

//     var material: Material = material_defaults();
//     material.base_color = vec3<f32>(0.0);
//     material.roughness = 0.05;
//     // material.clearcoat = 0.1;
//     var color = vec3<f32>(0.01);
//     for (var i = 1u; i < lights.count; i = i + 1u) {
//         let light = lights.lights[i];
//         let L = normalize(light.direction.xyz);

//         let shadow: f32 = 1.0;

//         let b = max(vec3<f32>(0.0), disney_brdf(material, L, V, N, X, Y));
//         color = color + shadow * dot(L, N) * light.color.rgb * b;
//     }
//     return vec4<f32>(color, 1.0);
// }

struct CubeEdgeFragmentOutput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] bright_color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_cube_edge_main(in: CubeEdgeVertexOutput) -> CubeEdgeFragmentOutput {
    let N = normalize(in.world_normal);
    let V = normalize(camera.view_position.xyz - in.world_position.xyz);

    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (USE_BRDF_FLAG) {
        color = edge_color_brdf(N, V, in.world_position);
    } else {
        color = edge_color_classic(N, V, in.world_position);
    }
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    var bright_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (brightness > 1.0) {
        bright_color = vec4<f32>(color.rgb, 1.0);
    }
    var out: CubeFaceFragmentOutput;
    out.color = color;
    out.bright_color = bright_color;
    return out;
}


// ----  Floor Fragment Shader   ---- ---- ---- ---- ---- ---- ---- ----

[[group(0), binding(3)]]
var t_floor_decal: texture_2d<f32>;
[[group(0), binding(4)]]
var s_floor_decal: sampler;

// No base material color.  It comes from the decal texture.
let floor_material_roughness: f32 = 0.6;

fn floor_color_brdf(
    tex_coord: vec2<f32>,
    N: vec3<f32>,
    V: vec3<f32>,
    world_pos: vec4<f32>,
) -> vec4<f32> {
    let X = normalize(cross(N, vec3<f32>(1.0, 0.0, 0.0))); // arbitrary
    let Y = normalize(cross(N, X));

    let material_color =
        textureSample(t_floor_decal, s_floor_decal, tex_coord).rgb * 0.3;

    var material = material_defaults();
    material.base_color = material_color * 3.0;
    material.roughness = 0.9;

    var color = vec3<f32>(0.0);

    // Ambient (cheating)
    color = color + lights.lights[0].color.rgb * material_color;

    // Directional lights
    for (var i = 1u; i < lights.count; i = i + 1u) {
        let light = lights.lights[i];
        let L = normalize(light.direction.xyz);

        let shadow = fetch_shadow16(i, light.proj * world_pos);

        let b = max(vec3<f32>(0.0), disney_brdf(material, L, V, N, X, Y));
        color = color + shadow * dot(L, N) * light.color.rgb * b;
    }
    return vec4<f32>(color, 1.0);
}

fn floor_color_classic(
    tex_coord: vec2<f32>,
    N: vec3<f32>,
    V: vec3<f32>,
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
        let L = normalize(light.direction.xyz);
        let H = normalize(V + L);

        let shadow = fetch_shadow16(i, light.proj * world_pos);

        let diffuse = lambert_diffuse(light.color.rgb, N, L);
        // let diffuse = burley_diffuse(floor_material_roughness, N, L, V, H);

        let specular = blinn_phong_specular(light.color.rgb, N, L, V, N);

        color = color + material_color * shadow * (diffuse + specular);
    }
    return vec4<f32>(color, 1.0);
}

struct FloorFragmentOutput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] bright_color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_floor_main(in: FloorVertexOutput) -> FloorFragmentOutput {
    let t_coord = vec2<f32>(in.decal_coords.x, 1.0 - in.decal_coords.y);

    let N = normalize(in.world_normal);
    let V = normalize(camera.view_position.xyz - in.world_position.xyz);

    var color: vec4<f32> = vec4<f32>(0.0);
    if (USE_BRDF_FLAG) {
        color = floor_color_brdf(t_coord, N, V, in.world_position);
    } else {
        color = floor_color_classic(t_coord, N, V, in.world_position);
    }
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    var bright_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (brightness > 1.0) {
        bright_color = vec4<f32>(color.rgb, 1.0);
    }
    var out: FloorFragmentOutput;
    out.color = color;
    out.bright_color = bright_color;
    return out;
}
