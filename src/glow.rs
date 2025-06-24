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
use wgpu::util::DeviceExt;

#[allow(dead_code)]
enum ResamplingAlgorithm {
    Lanczos, // most expensive, most accurate
    Boxes,   // cheap and cheesy
    Gapped,  // compromise
}
const RESAMPLING_ALGORITHM: ResamplingAlgorithm = ResamplingAlgorithm::Gapped;

const FACE_COUNT: usize = 6;
const CHANNEL_COUNT: usize = 4;

const SRC_FACE_WIDTH: usize = 64;
const SRC_FACE_HEIGHT: usize = 64;
const SRC_WIDTH: usize = FACE_COUNT * SRC_FACE_WIDTH;
const SRC_HEIGHT: usize = 1 * SRC_FACE_HEIGHT;
const SRC_BYTE_COUNT: usize = SRC_HEIGHT * SRC_WIDTH * CHANNEL_COUNT;
const SRC_FACE_BYTE_COUNT: usize =
    SRC_FACE_HEIGHT * SRC_FACE_WIDTH * CHANNEL_COUNT;

const INT_FACE_WIDTH: usize = 16;
const INT_FACE_HEIGHT: usize = 16;
const INT_WIDTH: usize = FACE_COUNT * INT_FACE_WIDTH;
const INT_HEIGHT: usize = 1 * INT_FACE_HEIGHT;
const INT_BYTE_COUNT: usize = INT_HEIGHT * INT_WIDTH * CHANNEL_COUNT;
const INT_FACE_BYTE_COUNT: usize =
    INT_FACE_HEIGHT * INT_FACE_WIDTH * CHANNEL_COUNT;

const DST_FACE_WIDTH: usize = 5;
const DST_FACE_HEIGHT: usize = 5;
const DST_WIDTH: usize = FACE_COUNT * DST_FACE_WIDTH;
const DST_HEIGHT: usize = 1 * DST_FACE_HEIGHT;
const DST_BYTE_COUNT: usize = DST_HEIGHT * DST_WIDTH * CHANNEL_COUNT;
const DST_FACE_BYTE_COUNT: usize =
    DST_FACE_HEIGHT * DST_FACE_WIDTH * CHANNEL_COUNT;

type SrcPixelArray = [u8; SRC_BYTE_COUNT];
type IntPixelArray = [u8; INT_BYTE_COUNT];
pub type DstPixelArray = [u8; DST_BYTE_COUNT];
type SrcFacePixelArray = [u8; SRC_FACE_BYTE_COUNT];
type IntFacePixelArray = [u8; INT_FACE_BYTE_COUNT];
type DstFacePixelArray = [u8; DST_FACE_BYTE_COUNT];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlowUniformRaw {
    face_xforms: [[[f32; 4]; 4]; 6],
}

pub struct Glow {
    uniform_buffer: wgpu::Buffer,
    glow_texture: wgpu::Texture,
    glow_view: wgpu::TextureView,
    resampler: Resampler,
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
                width: DST_WIDTH as _,
                height: DST_HEIGHT as _,
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

        let resampler = {
            let algorithm = RESAMPLING_ALGORITHM;
            let resizer = fir::Resizer::new();
            let data = [0u8; DST_BYTE_COUNT];
            Resampler {
                algorithm,
                resizer,
                data,
            }
        };

        Self {
            uniform_buffer,
            glow_texture,
            glow_view,
            resampler,
        }
    }

    pub fn uniform_resource(&self) -> wgpu::BindingResource {
        self.uniform_buffer.as_entire_binding()
    }

    pub fn glow_view_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.glow_view)
    }

    pub fn update(&mut self, blinky: &SrcPixelArray) {
        self.resampler.resample(blinky);
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
            &self.resampler.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(
                    (DST_WIDTH * CHANNEL_COUNT) as _,
                ),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: DST_WIDTH as _,
                height: DST_HEIGHT as _,
                depth_or_array_layers: 1,
            },
        );
    }
}

struct Resampler {
    algorithm: ResamplingAlgorithm,
    resizer: fir::Resizer,
    data: DstPixelArray,
}

impl Resampler {
    fn resample(&mut self, blinky: &SrcPixelArray) {
        match self.algorithm {
            ResamplingAlgorithm::Lanczos => self.resample_lanczos(blinky),
            ResamplingAlgorithm::Boxes => self.resample_boxes(blinky),
            ResamplingAlgorithm::Gapped => self.resample_gapped(blinky),
        };
    }

    fn resample_lanczos(&mut self, blinky: &SrcPixelArray) {
        for face in 0..FACE_COUNT {
            let mut src_face_bytes = blinky.get_face(face);
            let src_image = fir::images::Image::from_slice_u8(
                SRC_FACE_WIDTH as _,
                SRC_FACE_HEIGHT as _,
                &mut src_face_bytes,
                fir::PixelType::U8x4,
            )
            .unwrap();
            let mut dst_face_image = fir::images::Image::new(
                DST_FACE_WIDTH as _,
                DST_FACE_HEIGHT as _,
                fir::PixelType::U8x4);

            let options = fir::ResizeOptions::new()
                .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));

            self.resizer.resize(&src_image, &mut dst_face_image, &options).unwrap();

            let face_bytes = dst_face_image.buffer();
            self.data.set_face(face, face_bytes.try_into().unwrap());
        }
    }

    fn resample_boxes(&mut self, blinky: &SrcPixelArray) {
        const SCALE: usize = SRC_WIDTH / DST_WIDTH;
        assert!(SCALE * DST_WIDTH == SRC_WIDTH);
        const SRC_COL_BYTES: usize = CHANNEL_COUNT;
        const SRC_ROW_BYTES: usize = SRC_WIDTH * SRC_COL_BYTES;
        const DST_COL_BYTES: usize = CHANNEL_COUNT;
        const DST_ROW_BYTES: usize = DST_WIDTH * DST_COL_BYTES;
        for row in 0..DST_HEIGHT {
            let row_offset = SRC_ROW_BYTES * SCALE * row;
            for col in 0..DST_WIDTH {
                let row_col_offset = row_offset + SCALE * CHANNEL_COUNT * col;
                let mut accum = [0usize; 3];
                for i in 0..SCALE {
                    let i_offset = row_col_offset + i * SRC_ROW_BYTES;
                    for j in 0..SCALE {
                        accum[0] +=
                            blinky[i_offset + j * CHANNEL_COUNT + 0] as usize;
                        accum[1] +=
                            blinky[i_offset + j * CHANNEL_COUNT + 1] as usize;
                        accum[2] +=
                            blinky[i_offset + j * CHANNEL_COUNT + 2] as usize;
                    }
                }
                let s2 = SCALE * SCALE;
                self.data[row * DST_ROW_BYTES + col * DST_COL_BYTES + 0] =
                    (accum[0] / s2) as u8;
                self.data[row * DST_ROW_BYTES + col * DST_COL_BYTES + 1] =
                    (accum[1] / s2) as u8;
                self.data[row * DST_ROW_BYTES + col * DST_COL_BYTES + 2] =
                    (accum[2] / s2) as u8;
                self.data[row * DST_ROW_BYTES + col * DST_COL_BYTES + 3] = 255;
            }
        }
    }

    fn resample_gapped(&mut self, blinky: &SrcPixelArray) {
        // box-convert down to 16x16 (factor of 4x4).
        let int_bytes = self.resample_boxes_intermediate(blinky);

        // lanczos-convert down to 4x4 (factor of 4x4).
        self.resample_intermediate_lanczos(&int_bytes);
    }

    fn resample_boxes_intermediate(
        &mut self,
        blinky: &SrcPixelArray,
    ) -> IntPixelArray {
        const SCALE: usize = SRC_WIDTH / INT_WIDTH;
        assert!(SCALE * INT_WIDTH == SRC_WIDTH);
        const SRC_COL_BYTES: usize = CHANNEL_COUNT;
        const SRC_ROW_BYTES: usize = SRC_WIDTH * SRC_COL_BYTES;
        const INT_COL_BYTES: usize = CHANNEL_COUNT;
        const INT_ROW_BYTES: usize = INT_WIDTH * INT_COL_BYTES;
        let mut int_bytes = [0u8; INT_BYTE_COUNT];
        for row in 0..INT_HEIGHT {
            let row_offset = SRC_ROW_BYTES * SCALE * row;
            for col in 0..INT_WIDTH {
                let row_col_offset = row_offset + SCALE * CHANNEL_COUNT * col;
                let mut accum = [0usize; 3];
                for i in 0..SCALE {
                    let i_offset = row_col_offset + i * SRC_ROW_BYTES;
                    for j in 0..SCALE {
                        accum[0] +=
                            blinky[i_offset + j * CHANNEL_COUNT + 0] as usize;
                        accum[1] +=
                            blinky[i_offset + j * CHANNEL_COUNT + 1] as usize;
                        accum[2] +=
                            blinky[i_offset + j * CHANNEL_COUNT + 2] as usize;
                    }
                }
                let s2 = SCALE * SCALE;
                int_bytes[row * INT_ROW_BYTES + col * INT_COL_BYTES + 0] =
                    (accum[0] / s2) as u8;
                int_bytes[row * INT_ROW_BYTES + col * INT_COL_BYTES + 1] =
                    (accum[1] / s2) as u8;
                int_bytes[row * INT_ROW_BYTES + col * INT_COL_BYTES + 2] =
                    (accum[2] / s2) as u8;
                int_bytes[row * INT_ROW_BYTES + col * INT_COL_BYTES + 3] = 255;
            }
        }
        int_bytes
    }

    fn resample_intermediate_lanczos(&mut self, int_bytes: &IntPixelArray) {
        for face in 0..FACE_COUNT {
            let mut int_face_bytes = int_bytes.get_face(face);
            let int_image = fir::images::Image::from_slice_u8(
                INT_FACE_WIDTH as _,
                INT_FACE_HEIGHT as _,
                &mut int_face_bytes,
                fir::PixelType::U8x4,
            )
            .unwrap();
            let mut dst_face_image = fir::images::Image::new(
                DST_FACE_WIDTH as _,
                DST_FACE_HEIGHT as _,
                fir::PixelType::U8x4,
            );

            let options = fir::ResizeOptions::new()
                .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
            self.resizer.resize(&int_image, &mut dst_face_image, &options).unwrap();

            let face_bytes = dst_face_image.buffer();
            self.data.set_face(face, face_bytes.try_into().unwrap());
        }
    }
}

trait FaceSource<T> {
    fn get_face(&self, face: usize) -> T;
}

impl FaceSource<SrcFacePixelArray> for SrcPixelArray {
    fn get_face(&self, face: usize) -> SrcFacePixelArray {
        let mut data: SrcFacePixelArray = [0u8; SRC_FACE_BYTE_COUNT];
        let face_offset = face * SRC_FACE_WIDTH * CHANNEL_COUNT;
        for row in 0..SRC_FACE_HEIGHT {
            let row_offset = row * SRC_WIDTH * CHANNEL_COUNT;
            let face_row_offset = row * SRC_FACE_WIDTH * CHANNEL_COUNT;
            for col in 0..SRC_FACE_WIDTH {
                let col_offset = col * CHANNEL_COUNT;
                for chan in 0..CHANNEL_COUNT {
                    let i = face_offset + row_offset + col_offset + chan;
                    let j = face_row_offset + col_offset + chan;
                    data[j] = self[i];
                }
            }
        }
        data
    }
}

impl FaceSource<IntFacePixelArray> for IntPixelArray {
    fn get_face(&self, face: usize) -> IntFacePixelArray {
        let mut data: IntFacePixelArray = [0u8; INT_FACE_BYTE_COUNT];
        let face_offset = face * INT_FACE_WIDTH * CHANNEL_COUNT;
        for row in 0..INT_FACE_HEIGHT {
            let row_offset = row * INT_WIDTH * CHANNEL_COUNT;
            let face_row_offset = row * INT_FACE_WIDTH * CHANNEL_COUNT;
            for col in 0..INT_FACE_WIDTH {
                let col_offset = col * CHANNEL_COUNT;
                for chan in 0..CHANNEL_COUNT {
                    let i = face_offset + row_offset + col_offset + chan;
                    let j = face_row_offset + col_offset + chan;
                    data[j] = self[i];
                }
            }
        }
        data
    }
}

trait FaceDestination<T> {
    fn set_face(&mut self, face: usize, other: &T);
}

impl FaceDestination<DstFacePixelArray> for DstPixelArray {
    fn set_face(&mut self, face: usize, other: &DstFacePixelArray) {
        let face_offset = face * DST_FACE_WIDTH * CHANNEL_COUNT;
        for row in 0..DST_FACE_HEIGHT {
            let row_offset = row * DST_WIDTH * CHANNEL_COUNT;
            let face_row_offset = row * DST_FACE_WIDTH * CHANNEL_COUNT;
            for col in 0..DST_FACE_WIDTH {
                let col_offset = col * CHANNEL_COUNT;
                for chan in 0..CHANNEL_COUNT {
                    let i = face_offset + row_offset + col_offset + chan;
                    let j = face_row_offset + col_offset + chan;
                    self[i] = other[j];
                }
            }
        }
    }
}
