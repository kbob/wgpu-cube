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

type _P3 = cgmath::Point3<f32>;
type V3 = cgmath::Vector3<f32>;
type M4 = cgmath::Matrix4<f32>;

const MAX_LIGHTS: usize = 8;

const _WORLD_BOUND_MIN: V3 = cgmath::vec3::<f32>(-100.0, -100.0, 100.0);
const _WORLD_BOUND_MAX: V3 = cgmath::vec3::<f32>(100.0, 100.0, 1000.0);
const _WORLD_BOUNDS: cgmath::Ortho<f32> = cgmath::Ortho::<f32> {
    left: -100.0,
    right: 100.0,
    bottom: -100.0,
    top: 100.0,
    near: 1000.0,
    far: 100.0,
};

fn _round_up(n: usize, align: u32) -> usize {
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
        match self {
            Self::Ambient { intensity, color } => LightRaw {
                color: (color * *intensity).extend(1.0).into(),
                direction: [0.0, 0.0, 0.0, 0.0],
                position: [0.0, 0.0, 0.0, 0.0],
                proj: M4::zero().into(),
            },
            Self::Directional {
                intensity,
                color,
                direction,
            } => LightRaw {
                color: (color * *intensity).extend(1.0).into(),
                direction: direction.extend(1.0).into(),
                position: [0.0, 0.0, 0.0, 0.0],
                proj: M4::zero().into(),
            },
        }
    }
}

struct _OoLight {
    intensity: f32,
    color: V3,
    direction: Option<V3>,
    position: Option<V3>,
    _fov: Option<f32>,
}

impl _OoLight {
    const fn _ambient(intensity: f32, color: V3) -> Self {
        Self {
            intensity,
            color,
            direction: None,
            position: None,
            _fov: None,
        }
    }
    const fn _directional(intensity: f32, color: V3, direction: V3) -> Self {
        Self {
            intensity,
            color,
            direction: Some(direction),
            position: None,
            _fov: None,
        }
    }
    const fn _point(intensity: f32, color: V3, position: V3) -> Self {
        Self {
            intensity,
            color,
            position: Some(position),
            direction: None,
            _fov: None,
        }
    }
    const fn _spot(
        intensity: f32,
        color: V3,
        position: V3,
        direction: V3,
        fov: f32,
    ) -> Self {
        Self {
            intensity,
            color,
            position: Some(position),
            direction: Some(direction),
            _fov: Some(fov),
        }
    }

    fn _is_ambient(&self) -> bool {
        self.direction.is_none() && self.position.is_none()
    }
    fn _is_directional(&self) -> bool {
        self.direction.is_some() && self.position.is_none()
    }
    fn _is_point(&self) -> bool {
        self.direction.is_none() && self.position.is_some()
    }
    fn _is_spot(&self) -> bool {
        self.direction.is_some() && self.position.is_some()
    }

    fn _to_raw(&self) -> LightRaw {
        let color = (self.intensity * self.color.extend(1.0)).into();
        let direction = match self.direction {
            Some(dir) => dir.extend(1.0).into(),
            None => [0.0, 0.0, 0.0, 0.0],
        };
        let position = match self.position {
            Some(pos) => pos.extend(1.0).into(),
            None => [0.0, 0.0, 0.0, 0.0],
        };
        let proj = M4::zero().into();

        LightRaw {
            color,
            direction,
            position,
            proj,
        }
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
    pub uniform_buffer: wgpu::Buffer,
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
        ];
        assert!(lights.len() <= MAX_LIGHTS);

        let uniform_raw: LightsUniformRaw = lights_to_raw(&lights);
        let uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lights_uniform_buffer"),
                contents: bytemuck::cast_slice(&[uniform_raw]),
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::UNIFORM,
            });

        Self {
            lights,
            uniform_buffer,
        }
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
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[prepared.lights_uniform]),
        );
    }
}
