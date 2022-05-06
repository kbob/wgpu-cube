// Vertex Shader

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] coord: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
) -> VertexOutput {
    let u = (position.x + 1.0) * 0.5;
    let v = (position.y - 1.0) * -0.5;
    let coord = vec2<f32>(u, v);
    return VertexOutput(position, coord);
}

// [[stage(vertex)]]
// fn vs_main([[location(0)]] pos: vec4<f32>) -> [[builtin(position)]] vec4<f32> {
//     return pos;
// }

// Fragment Shader

struct PostUniform {
    axis: u32;
};
[[group(0), binding(0)]]
var<uniform> post: PostUniform;

[[group(0), binding(1)]]
var t_image: texture_2d<f32>;

[[group(0), binding(2)]]
var s_image: sampler;

let offsets: array<f32, 3> = array<f32, 3>(0.0, 1.3846153846, 3.2307692308);
let weights: array<f32, 3> =
    array<f32, 3>(0.2270270270, 0.3162162162, 0.0702702703);

[[stage(fragment)]]
fn fs_horizontal_blur_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    let tex_coord = in.coord;
    let image_size = vec2<f32>(textureDimensions(t_image));

//     // // wgsl doesn't allow indexing an array with a variable???
//     // let off = vec2<f32>(offsets[0], 0.0);
//     // let wgt = weights[0];
//     // let coord = (tex_coord / image_size);
//     // let samp = textureSample(t_image, s_image, coord).rgb;
//     // var color = wgt * samp;
//     // for (var i = 1; i < 3; i = i + 1) {
//     //     let off = vec2<f32>(offsets[i], 0.0) / image_size;
//     //     let wgt = weights[i];
//     //     let coord_n = tex_coord - off;
//     //     let coord_p = tex_coord + off;
//     //     let samp_n = textureSample(t_image, s_image, coord_n).rgb;
//     //     let samp_p = textureSample(t_image, s_image, coord_p).rgb;
//     //     let color = color + wgt * (samp_n + samp_p);
//     // }
//     // return vec4<f32>(color, 1.0);

    // unroll.
    let off_0 = vec2<f32>(offsets[0] / image_size.x, 0.0);
    let off_1 = vec2<f32>(offsets[1], 0.0) / image_size;
    let off_2 = vec2<f32>(offsets[2], 0.0) / image_size;
    let weight_0 = weights[0];
    let weight_1 = weights[1];
    let weight_2 = weights[2];
    let coord_n2 = tex_coord - off_2;
    let coord_n1 = tex_coord - off_1;
    let coord_p0 = tex_coord + off_0;
    let coord_p1 = tex_coord + off_1;
    let coord_p2 = tex_coord + off_2;
    let samp_n2 = textureSample(t_image, s_image, coord_n2).rgb;
    let samp_n1 = textureSample(t_image, s_image, coord_n1).rgb;
    let samp_p0 = textureSample(t_image, s_image, coord_p0).rgb;
    let samp_p1 = textureSample(t_image, s_image, coord_p1).rgb;
    let samp_p2 = textureSample(t_image, s_image, coord_p2).rgb;
    let color_0 = weight_0 * samp_p0;
    let color_1 = weight_1 * (samp_n1 + samp_p1);
    let color_2 = weight_2 * (samp_n2 + samp_p2);
    let color = color_0 + color_1 + color_2;
    return vec4<f32>(color, 1.0);
}

[[stage(fragment)]]
fn fs_vertical_blur_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    let tex_coord = in.coord;
    let image_size = vec2<f32>(textureDimensions(t_image));

    // // wgsl doesn't allow indexing an array with a variable???
    // let off = vec2<f32>(0.0, offsets[0]);
    // let wgt = weights[0];
    // let coord = (tex_coord / image_size);
    // let samp = textureSample(t_image, s_image, coord).rgb;
    // var color = wgt * samp;
    // for (var i = 1; i < 3; i = i + 1) {
    //     let off = vec2<f32>(0.0, offsets[i]);
    //     let wgt = weights[i];
    //     let coord_n = (tex_coord - off) / image_size;
    //     let coord_p = (tex_coord + off) / image_size;
    //     let samp_n = textureSample(t_image, s_image, coord_n).rgb;
    //     let samp_p = textureSample(t_image, s_image, coord_p).rgb;
    //     let color = color + wgt * (samp_n + samp_p);
    // }
    // return vec4<f32>(color, 1.0);

    // unroll.
    let off_0 = vec2<f32>(0.0, offsets[0]) / image_size;
    let off_1 = vec2<f32>(0.0, offsets[1]) / image_size;
    let off_2 = vec2<f32>(0.0, offsets[2]) / image_size;
    let weight_0 = weights[0];
    let weight_1 = weights[1];
    let weight_2 = weights[2];
    let coord_n2 = tex_coord - off_2;
    let coord_n1 = tex_coord - off_1;
    let coord_p0 = tex_coord + off_0;
    let coord_p1 = tex_coord + off_1;
    let coord_p2 = tex_coord + off_2;
    let samp_n2 = textureSample(t_image, s_image, coord_n2).rgb;
    let samp_n1 = textureSample(t_image, s_image, coord_n1).rgb;
    let samp_p0 = textureSample(t_image, s_image, coord_p0).rgb;
    let samp_p1 = textureSample(t_image, s_image, coord_p1).rgb;
    let samp_p2 = textureSample(t_image, s_image, coord_p2).rgb;
    let color_0 = weight_0 * samp_p0;
    let color_1 = weight_1 * (samp_n1 + samp_p1);
    let color_2 = weight_2 * (samp_n2 + samp_p2);
    let color = color_0 + color_1 + color_2;
    return vec4<f32>(color, 1.0);
}

let EXPOSURE: f32 = 1.0;
let GAMMA: f32 = 2.2;

[[group(0), binding(3)]]
var t_bright: texture_2d<f32>;

[[group(0), binding(4)]]
var s_bright: sampler;

[[stage(fragment)]]
fn fs_composite_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    let coord = in.coord;
    let image_size = vec2<f32>(textureDimensions(t_image));

    let index = vec2<i32>(in.position.xy + 0.5);
    let ldr_color = textureLoad(t_image, index, 0).rgb;
    let bright_color = textureLoad(t_bright, index, 0).rgb;
    let hdr_color = ldr_color + bright_color;

    // tone_mapping
    let mapped_color = vec3<f32>(1.0) - exp(-hdr_color * EXPOSURE);

    // correct gamma
    // let gc_color = pow(mapped_color, vec3<f32>(1.0 / GAMMA));
    let gc_color = mapped_color;

    return vec4<f32>(gc_color, 1.0);
}
