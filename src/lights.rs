use wgpu::util::*;

use crate::prelude::*;
use crate::traits::Renderable;

// Light types
//  - ambient
//  - directional
//  - point
//  - spot
//  - ... other
// Should probably start with directional.
//
// Light fields:
//  - intensity (all)
//  - color (all)
//  - direction (directional, spot)
//  - position (point, spot)
//  - fov (spot)

pub const MAX_LIGHTS: usize = 8;

const SHADOW_MAP_SIZE: u32 = 128;
pub const SHADOW_MAP_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Depth32Float;

fn round_up(n: usize, align: u32) -> usize {
    let align = align as usize;
    (n + align - 1) / align * align
}

// return the corners of an Ortho used as a bounding box.
fn ortho_corners(orth: &cgmath::Ortho<f32>) -> Vec<Vec3> {
    let mut corners = vec![];
    for x in [orth.left, orth.right] {
        for y in [orth.bottom, orth.top] {
            for z in [orth.far, orth.near] {
                corners.push((x, y, z).into());
            }
        }
    }

    corners
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LightRaw {
    color: [f32; 4],
    direction: [f32; 4],
    position: [f32; 4],
    proj: [[f32; 4]; 4],
    shadow_map_size: f32,
    shadow_map_inv_size: f32,
    _padding: [i32; 2],
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
        color: Vec3,
    },
    Directional {
        intensity: f32,
        color: Vec3,
        direction: Vec3,
    },
    // Point { intensity: f32, color: Vec3, position: Point3 },
    // Spot {
    //     intensity: f32,
    //     color: Vec3,
    //     direction: Vec3,
    //     position: Point3,
    //     fov: f32,
    // },
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
                shadow_map_size: 1f32,
                shadow_map_inv_size: 1f32,
                _padding: [0, 0],
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
                shadow_map_size: SHADOW_MAP_SIZE as f32,
                shadow_map_inv_size: 1.0 / SHADOW_MAP_SIZE as f32,
                _padding: [0, 0],
            },
        }
    }
    fn create_projection(&self) -> Mat4 {
        match self {
            Self::Ambient { .. } => Mat4::zero(),
            Self::Directional { direction: dir, .. } => self.create_ortho(dir),
        }
    }

    fn create_ortho(&self, dir: &Vec3) -> Mat4 {
        // rotation matrix looks away from the light
        let away_from_light =
            Mat4::look_to_rh(Point3::origin(), -*dir, Vec3::unit_y());

        // ortho projection matrix is wide enough to hold the cube,
        // but `far` is extended to reach the furthest corner of
        // the floor.
        let mut bounds = crate::cube::CUBE_BOUNDS_WORLD;
        bounds.near = -bounds.near; // near/far are distance to, not
        bounds.far = -bounds.far; // coordinates.
        bounds.far = ortho_corners(&crate::floor::FLOOR_BOUNDS_WORLD)
            .iter()
            .map(|corner| -(away_from_light * corner.extend(1.0)).z)
            .fold(bounds.far, |max, z| max.max(z));
        let ortho: Mat4 = bounds.into();

        crate::camera::OPENGL_TO_WGPU_MATRIX * ortho * away_from_light
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
                color: Vec3::new(1.0, 1.0, 1.0),
            },
            Light::Directional {
                intensity: 1.0,
                color: Vec3::new(1.0, 0.7, 0.5),
                direction: Vec3::new(-1.0, 1.0, 1.0),
            },
            Light::Directional {
                intensity: 1.0,
                color: Vec3::new(1.0, 0.9, 1.0),
                direction: Vec3::new(-1.0, 1.0, -0.2),
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
            buffer.slice(..).get_mapped_range_mut()[..buffer_size]
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
                std::mem::size_of::<ShadowUniformRaw>() as _,
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
