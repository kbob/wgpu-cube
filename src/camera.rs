use crate::prelude::*;
use wgpu::util::DeviceExt;

use crate::traits::Renderable;
use crate::Hand;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0, -0.5,  0.0,
    0.0,  0.0,  0.5,  1.0,
);
#[rustfmt::skip]
pub const LEFT_HAND_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0,  0.5,  0.0,
    0.0,  0.0,  0.5,  1.0,
);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniformRaw {
    view_position: [f32; 4],
    world_to_clip: [[f32; 4]; 4],
    framebuffer_to_texture: [f32; 2],
    _padding: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub width: u32,
    pub height: u32,
}

pub struct Camera {
    config: Configuration,
    eye: Point3,
    target: Point3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    world_hand: Hand,

    uniform_buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        config: &Configuration,
        world_hand: Hand,
    ) -> Self {
        let f2p: [f32; 2] = [
            1.0 / config.width as f32,
            1.0 / config.height as f32,
        ];
        let uniform_raw = CameraUniformRaw {
            view_position: [0.0, 0.0, 0.0, 0.0],
            world_to_clip: Mat4::identity().into(),
            framebuffer_to_texture: f2p,
            _padding: [0.0, 0.0],
        };
        let uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera_uniform_buffer"),
                contents: bytemuck::cast_slice(&[uniform_raw]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        let backup = 1.0; // debug: increase to back up.
        Self {
            config: *config,
            // hardcoded position, oh my!
            eye: (0.0, backup * 170.0, backup * 300.0).into(),
            target: (60.0, 0.0, 0.0).into(),
            up: Vec3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 100.0,
            zfar: backup * 1000.0,
            world_hand: world_hand,
            uniform_buffer,
        }
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.build_view_projection_matrix()
    }

    pub fn uniform_resource(&self) -> wgpu::BindingResource {
        self.uniform_buffer.as_entire_binding()
    }

    pub fn resize(&mut self, config: &Configuration) {
        self.aspect = config.width as f32 / config.height as f32;
        self.config = *config;
    }

    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
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

pub struct CameraAttributes();

pub struct CameraPreparedData {
    camera_uniform: CameraUniformRaw,
}

impl Renderable<CameraAttributes, CameraPreparedData> for Camera {
    fn prepare(&self, _: &CameraAttributes) -> CameraPreparedData {
        let f2p: [f32; 2] = [
            1.0 / self.config.width as f32,
            1.0 / self.config.height as f32,
        ];
        CameraPreparedData {
            camera_uniform: CameraUniformRaw {
                view_position: self.eye.to_homogeneous().into(),
                world_to_clip: self.build_view_projection_matrix().into(),
                framebuffer_to_texture: f2p,
                _padding: [0.0, 0.0],
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
