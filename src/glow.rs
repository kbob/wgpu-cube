//  uniform:
//      cube transform (already there)
//      face transforms
//  texture:
//      small blinky
//
//  The illumination model is pretty much the same:
//      if light is visible:
//          calc BRDF

use crate::prelude::*;
use crate::traits::Renderable;
use fast_image_resize as fir;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

const FACE_COUNT: usize = 6;
const CHANNEL_COUNT: usize = 4;

const SRC_FACE_WIDTH: usize = 64;
const SRC_FACE_HEIGHT: usize = 64;
const SRC_WIDTH: usize = FACE_COUNT * SRC_FACE_WIDTH;
const SRC_HEIGHT: usize = 1 * SRC_FACE_HEIGHT;

const DST_FACE_WIDTH: usize = 4;
const DST_FACE_HEIGHT: usize = 4;
const DST_WIDTH: usize = FACE_COUNT * DST_FACE_WIDTH;
const DST_HEIGHT: usize = 1 * DST_FACE_HEIGHT;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlowUniformRaw {
    face_xforms: [[[f32; 4]; 4]; 6],
}

pub struct Glow {
    uniform_buffer: wgpu::Buffer,
    glow_texture: wgpu::Texture,
    glow_view: wgpu::TextureView,
    resizer: fir::Resizer,
    data: [u8; 4 * 4 * 6 * 4],
}

impl Glow {
    pub fn new(device: &wgpu::Device, face_xforms: &Vec<Mat4>) -> Self {
        let face_xform_vec: Vec<[[f32; 4]; 4]> =
            face_xforms.iter().map(|xf| Mat4::into(*xf)).collect();

        let uniform_data = GlowUniformRaw {
            face_xforms: face_xform_vec.try_into().unwrap(),
        };

        let uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("glow_uniform"),
                contents: bytemuck::cast_slice(&[uniform_data]),
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::UNIFORM,
            });

        let glow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glow_texture"),
            size: wgpu::Extent3d {
                width: 24,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let glow_view =
            glow_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("glow_texture_view"),
                ..Default::default()
            });
        let resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(
            fir::FilterType::Lanczos3,
        ));

        let data = [0u8; 4 * 6 * 4 * 4];

        Self {
            uniform_buffer,
            glow_texture,
            glow_view,
            resizer,
            data,
        }
    }

    pub fn uniform_resource(&self) -> wgpu::BindingResource {
        self.uniform_buffer.as_entire_binding()
    }

    pub fn glow_view_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.glow_view)
    }

    pub fn update(&mut self, blinky: &[u8; 6 * 64 * 64 * 4]) {
        // Do the image resampling thing here.
        let mut src_bytes = blinky.clone();
        let src_image = fir::Image::from_slice_u8(
            NonZeroU32::new(SRC_WIDTH as _).unwrap(),
            NonZeroU32::new(SRC_HEIGHT as _).unwrap(),
            &mut src_bytes,
            fir::PixelType::U8x4,
        )
        .unwrap();
        let mut src_view = src_image.view();
        for face in 0..FACE_COUNT {
            src_view
                .set_crop_box(fir::CropBox {
                    left: (face * SRC_FACE_WIDTH) as u32,
                    top: 0,
                    width: NonZeroU32::new(SRC_FACE_WIDTH as _).unwrap(),
                    height: NonZeroU32::new(SRC_FACE_HEIGHT as _).unwrap(),
                })
                .unwrap();
            let mut dst_face_image = fir::Image::new(
                NonZeroU32::new(DST_FACE_WIDTH as _).unwrap(),
                NonZeroU32::new(DST_FACE_HEIGHT as _).unwrap(),
                fir::PixelType::U8x4,
            );
            let mut dst_view = dst_face_image.view_mut();

            self.resizer.resize(&src_view, &mut dst_view).unwrap();

            let face_bytes = dst_face_image.buffer();

            self.copy_face(face, face_bytes);
        }
    }

    fn copy_face(&mut self, face: usize, bytes: &[u8]) {
        let face_offset = face * DST_FACE_WIDTH * CHANNEL_COUNT;
        for row in 0..DST_HEIGHT {
            let face_row_bytes = DST_FACE_WIDTH * CHANNEL_COUNT;
            let src_row_offset = row * face_row_bytes;
            let dst_row_offset = face_offset + row * DST_WIDTH * CHANNEL_COUNT;
            self.data[dst_row_offset..dst_row_offset + face_row_bytes]
                .copy_from_slice(
                    &bytes[src_row_offset..src_row_offset + face_row_bytes],
                );
        }
    }
}

pub struct GlowAttributes();
pub struct GlowPreparedData();

impl Renderable<GlowAttributes, GlowPreparedData> for Glow {
    fn prepare(&self, _: &GlowAttributes) -> GlowPreparedData {
        GlowPreparedData()
    }

    fn render<'rpass>(
        &'rpass self,
        queue: &wgpu::Queue,
        _: &mut wgpu::RenderPass<'rpass>,
        _: &'rpass GlowPreparedData,
    ) {
        queue.write_texture(
            self.glow_texture.as_image_copy(),
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(6 * 4 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 6 * 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );
    }
}
