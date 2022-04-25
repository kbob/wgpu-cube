use cgmath::prelude::*;
use wgpu::util::*;

use crate::traits::Renderable;

// Light types
//  - ambient
//  - directional
//  - point
//  - spot
//  - ... other
// Should probably start with point.
//
// Light fields:
//  - intensity (all)
//  - color (all)
//  - direction (directional, spot)
//  - position (point, spot)
//  - fov (spot)

#[allow(dead_code)]
type P3 = cgmath::Point3<f32>;
#[allow(dead_code)]
type V3 = cgmath::Vector3<f32>;
#[allow(dead_code)]
type V4 = cgmath::Vector4<f32>;
#[allow(dead_code)]
type M3 = cgmath::Matrix3<f32>;
#[allow(dead_code)]
type M4 = cgmath::Matrix4<f32>;

pub const MAX_LIGHTS: usize = 8;

// const _WORLD_BOUND_MIN: V3 = V3(-100.0, -100.0, 100.0);
// const _WORLD_BOUND_MAX: V3 = V3(100.0, 100.0, 1000.0);
const WORLD_BOUNDS: cgmath::Ortho<f32> = cgmath::Ortho::<f32> {
    left: -260.0,
    right: 260.0,
    bottom: -101.0,
    top: 100.0,
    near: 100.0,
    far: -600.0,
};

const SHADOW_MAP_SIZE: u32 = 512;
const SHADOW_MAP_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Depth32Float;

fn round_up(n: usize, align: u32) -> usize {
    let align = align as usize;
    (n + align - 1) / align * align
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LightRaw {
    color: [f32; 4],
    direction: [f32; 4],
    position: [f32; 4],
    proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightsUniformRaw {
    count: u32,
    _padding: [u32; 3],
    lights: [LightRaw; MAX_LIGHTS],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowUniformRaw {
    proj: [[f32; 4]; 4],
}

enum Light {
    Ambient {
        intensity: f32,
        color: V3,
    },
    Directional {
        intensity: f32,
        color: V3,
        direction: V3,
    },
    // Point { intensity: f32, color: V3, position: P3 },
    //Spot { intensity: f32, color: V3, direction: V3, position: P3, fov: f32 },
}

impl Light {
    fn to_raw(&self) -> LightRaw {
        let proj: [[f32; 4]; 4] = self.create_projection().into();
        match self {
            Self::Ambient { intensity, color } => LightRaw {
                color: (color * *intensity).extend(1.0).into(),
                direction: [0.0, 0.0, 0.0, 0.0],
                position: [0.0, 0.0, 0.0, 0.0],
                proj: proj,
            },
            Self::Directional {
                intensity,
                color,
                direction,
            } => LightRaw {
                color: (color * *intensity).extend(1.0).into(),
                direction: direction.extend(1.0).into(),
                position: [0.0, 0.0, 0.0, 0.0],
                proj: proj,
            },
        }
    }
    fn create_projection(&self) -> M4 {
        match self {
            Self::Ambient { .. } => M4::zero(),
            Self::Directional { direction: dir, .. } => self.create_ortho(dir),
        }
    }

    #[allow(unused_variables, unused_assignments, dead_code, unreachable_code)]
    fn create_ortho(&self, dir: &V3) -> M4 {

        let world_bounds: cgmath::Ortho<f32> = cgmath::Ortho::<f32> {
            left: -120.0,
            right: 120.0,
            bottom: -120.0,
            top: 120.0,
            // near: 1300.0,
            // far: -180.0,
            near: -1800.0,
            far: 1300.0,
        };
        let wbo: M4 = world_bounds.into();

        const CORRECTION: M4 = M4::from_cols(
            V4::new(1.0, 0.0, 0.0, 0.0),
            V4::new(0.0, 1.0, 0.0, 0.0),
            V4::new(0.0, 0.0, -1.0, 0.0),
            V4::new(0.0, 0.0, 1.0, 1.0),
        );

        let mut proj: M4 = M4::identity();

        proj = M4::look_to_rh(P3::origin(), -*dir, V3::unit_y()) * proj;
        proj = wbo * proj;
        proj = CORRECTION * proj;
        return proj;


        // Find the minimal box aligned with dir that contains
        // the 8 corners of the world.
        let b = &WORLD_BOUNDS;
        let pts = vec![
            (b.left, b.bottom, b.far),
            (b.left, b.bottom, b.near),
            (b.left, b.top, b.far),
            (b.left, b.top, b.near),
            (b.right, b.bottom, b.far),
            (b.right, b.bottom, b.near),
            (b.right, b.top, b.far),
            (b.right, b.top, b.near),
        ];
        let rotate = M3::look_to_rh(*dir, V3::unit_y());
        let empty_ortho = cgmath::Ortho::<f32> {
            left: f32::MAX,
            right: f32::MIN,
            bottom: f32::MAX,
            top: f32::MIN,
            far: f32::MAX,
            near: f32::MIN,
        };
        let light_bounds = pts.iter().fold(empty_ortho, |accum, item| {
            let pt: V3 = (*item).into();
            let pt: V3 = rotate * pt;
            cgmath::Ortho::<f32> {
                left: f32::min(accum.left, pt.x),
                right: f32::max(accum.right, pt.x),
                bottom: f32::min(accum.bottom, pt.y),
                top: f32::max(accum.top, pt.y),
                far: f32::min(accum.far, pt.z),
                near: f32::max(accum.near, pt.z),
            }
        });
        let proj: M4 = light_bounds.into();

        let mut proj: M4 = WORLD_BOUNDS.into();

        proj = M4::look_to_rh(P3::origin(), *dir, V3::unit_y()) * proj;
        proj
    }
}

fn lights_to_raw(light_vec: &Vec<Light>) -> LightsUniformRaw {
    assert!(light_vec.len() <= MAX_LIGHTS);
    let count = light_vec.len() as u32;
    let _padding = [0, 0, 0];
    let mut lights: [LightRaw; 8] = Default::default();
    for (i, light) in light_vec.iter().enumerate() {
        if i < lights.len() {
            lights[i] = light.to_raw();
        }
    }
    LightsUniformRaw {
        count,
        _padding,
        lights,
    }
}

pub struct Lights {
    lights: Vec<Light>,
    light_uniform_buffer: wgpu::Buffer,
    shadow_uniform_buffer: wgpu::Buffer,
    shadow_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    shadow_target_views: Vec<wgpu::TextureView>,
}

impl Lights {
    pub fn new(device: &wgpu::Device) -> Self {
        let lights = vec![
            Light::Ambient {
                intensity: 0.05,
                color: V3::new(1.0, 1.0, 1.0),
            },
            Light::Directional {
                intensity: 1.0,
                color: V3::new(1.0, 0.7, 0.5),
                direction: V3::new(-1.0, 1.0, 1.0),
            },
            Light::Directional {
                intensity: 1.0,
                color: V3::new(1.0, 0.9, 1.0),
                direction: V3::new(-1.0, 1.0, -0.2),
            },
        ];
        assert!(lights.len() <= MAX_LIGHTS);

        let uniform_raw: LightsUniformRaw = lights_to_raw(&lights);
        let light_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lights_uniform_buffer"),
                contents: bytemuck::cast_slice(&[uniform_raw]),
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::UNIFORM,
            });

        // Shadow Uniform Buffer

        let shadow_uniform_buffer = {
            let raw_size = std::mem::size_of::<ShadowUniformRaw>();
            let min_align = device.limits().min_uniform_buffer_offset_alignment;
            let aligned_size = round_up(raw_size, min_align);
            let buffer_size = lights.len() * aligned_size;

            let mut data = vec![0u8; buffer_size];

            for (i, light) in lights.iter().enumerate() {
                let proj = light.create_projection();
                let offset = i * aligned_size;
                let end = offset + raw_size;
                *bytemuck::from_bytes_mut::<ShadowUniformRaw>(
                    &mut data[offset..end],
                ) = ShadowUniformRaw { proj: proj.into() };
            }

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("shadow_uniform_buffer"),
                size: buffer_size as _,
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: true,
            });
            buffer
                .slice(..)
                .get_mapped_range_mut()[..buffer_size]
                .copy_from_slice(bytemuck::cast_slice(&data));
            buffer.unmap();

            buffer
        };

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_texture"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: lights.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SHADOW_MAP_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let shadow_view =
            shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::GreaterEqual),
            ..Default::default()
        });

        let shadow_target_views = (0..lights.len())
            .map(|i| {
                shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("light_shadow_view_{}", i)),
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: i as u32,
                    array_layer_count: std::num::NonZeroU32::new(1),
                })
            })
            .collect::<Vec<_>>();

        Self {
            lights,
            light_uniform_buffer,
            shadow_uniform_buffer,
            shadow_view,
            shadow_sampler,
            shadow_target_views,
        }
    }

    pub fn light_uniform_resource(&self) -> wgpu::BindingResource {
        self.light_uniform_buffer.as_entire_binding()
    }

    pub fn shadow_uniform_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.shadow_uniform_buffer,
            offset: 0,
            size: wgpu::BufferSize::new(
                std::mem::size_of::<ShadowUniformRaw>() as _
            ),
        })
    }

    pub fn shadow_maps_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.shadow_view)
    }

    pub fn shadow_maps_sampler_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.shadow_sampler)
    }

    pub fn light_has_shadow(&self, light_index: usize) -> bool {
        light_index < self.lights.len()
            && match self.lights[light_index] {
                Light::Ambient { .. } => false,
                _ => true,
            }
    }

    pub fn light_shadow_view(&self, light_index: usize) -> &wgpu::TextureView {
        &self.shadow_target_views[light_index]
    }

    pub fn shadow_uniform_offset(
        &self,
        device: &wgpu::Device,
        light_index: usize,
    ) -> wgpu::DynamicOffset {
        let raw_size = std::mem::size_of::<ShadowUniformRaw>();
        let min_align = device.limits().min_uniform_buffer_offset_alignment;
        let aligned_size = round_up(raw_size, min_align);
        (aligned_size * light_index) as wgpu::DynamicOffset
    }

    fn to_raw(&self) -> LightsUniformRaw {
        lights_to_raw(&self.lights)
    }
}

pub struct LightsAttributes();

pub struct LightsPreparedData {
    lights_uniform: LightsUniformRaw,
}

impl Renderable<LightsAttributes, LightsPreparedData> for Lights {
    fn prepare(&self, _: &LightsAttributes) -> LightsPreparedData {
        LightsPreparedData {
            lights_uniform: self.to_raw(),
        }
    }

    fn render<'rpass>(
        &'rpass self,
        queue: &wgpu::Queue,
        _render_pass: &mut wgpu::RenderPass<'rpass>,
        prepared: &'rpass LightsPreparedData,
    ) {
        queue.write_buffer(
            &self.light_uniform_buffer,
            0,
            bytemuck::cast_slice(&[prepared.lights_uniform]),
        );
    }
}
