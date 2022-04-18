use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::traits::Renderable;
use crate::Hand;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0, -0.5,  0.0,
    0.0,  0.0,  0.5,  1.0,
);
#[rustfmt::skip]
pub const LEFT_HAND_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0,  0.5,  0.0,
    0.0,  0.0,  0.5,  1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniformRaw {
    view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    world_hand: Hand,

    pub uniform_buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        world_hand: Hand,
    ) -> Self {
        let uniform_raw = CameraUniformRaw {
            view_proj: cgmath::Matrix4::<f32>::identity().into(),
        };
        let uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera_uniform_buffer"),
                contents: bytemuck::cast_slice(&[uniform_raw]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            // hardcoded position, oh my!
            eye: (100.0, 150.0, 200.0).into(),
            target: (60.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 1.0,
            zfar: 1000.0,
            world_hand: world_hand,
            uniform_buffer,
        }
    }
    pub fn set_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect,
            self.znear,
            self.zfar,
        );
        let convert = match self.world_hand {
            Hand::Left => LEFT_HAND_TO_WGPU_MATRIX,
            Hand::Right => OPENGL_TO_WGPU_MATRIX,
        };
        convert * proj * view
    }
}

pub struct CameraAttributes {}

pub struct CameraPreparedData {
    camera_uniform: CameraUniformRaw,
}

impl Renderable<CameraAttributes, CameraPreparedData> for Camera {
    fn prepare(&self, _: &CameraAttributes) -> CameraPreparedData {
        CameraPreparedData {
            camera_uniform: CameraUniformRaw {
                view_proj: self.build_view_projection_matrix().into(),
            },
        }
    }

    fn render<'rpass>(
        &'rpass self,
        queue: &wgpu::Queue,
        _render_pass: &mut wgpu::RenderPass<'rpass>,
        prepared: &'rpass CameraPreparedData,
    ) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[prepared.camera_uniform]),
        );
    }
}
