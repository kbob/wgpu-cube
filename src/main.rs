use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

mod cube;
mod cube_model;
mod texture;
mod traits;
mod trackball;
use traits::Renderable;
use trackball::{
    Manipulable,
    Responder,
};

const BACKFACE_CULL: bool = true;

#[allow(dead_code)]
#[derive(PartialEq)]
enum Hand {
    Left,
    Right,
}
const WORLD_HANDEDNESS: Hand = Hand::Right;

const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.00250, g: 0.00625, b: 0.01500, a: 1.0,
};

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

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect,
            self.znear,
            self.zfar,
        );
        let convert = match WORLD_HANDEDNESS {
            Hand::Left => LEFT_HAND_TO_WGPU_MATRIX,
            Hand::Right => OPENGL_TO_WGPU_MATRIX,
        };
        convert * proj * view
    }

    fn configure(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.aspect = config.width as f32 / config.height as f32;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

fn create_render_pipeline(
    label: &str,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&shader);

    device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: match BACKFACE_CULL {
                    true => Some(wgpu::Face::Back),
                    false => None,
                },
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format|
                wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare:
                        match WORLD_HANDEDNESS {
                            Hand::Left => wgpu::CompareFunction::Less,
                            Hand::Right => wgpu::CompareFunction::Greater,
                        },
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                ],
            }),
            multiview: None,
        }
    )
}

struct State {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_texture: texture::Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    _camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    cube: cube::Cube,
    cube_trackball: trackball::Trackball,
}

impl State {
    async fn new(window: &Window) -> Self {

        // Device and Surface

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // Camera

        let camera = Camera {
            eye: (100.0, 150.0, 300.0).into(),
            target: (60.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 1.0,
            zfar: 1000.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let _camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("_camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: (
                    wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST
                ),
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("camera_bind_group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: _camera_buffer.as_entire_binding(),
                    }
                ],
            }
        );

        // Cube Object

        let cube = cube::Cube::_new(
            &device,
            &queue,
            config.format,
            &camera_bind_group_layout,
        );

        let cube_trackball = trackball::Trackball::new(&size);

        // Pipeline

        let depth_texture = texture::Texture::create_depth_texture(
            "depth_texture",
            &device,
            &config,
            match WORLD_HANDEDNESS {
                Hand::Left => wgpu::CompareFunction::LessEqual,
                Hand::Right => wgpu::CompareFunction::GreaterEqual,
            },
        );

        // Results

        Self {
            size,
            surface,
            device,
            queue,
            config,
            depth_texture,
            camera,
            camera_uniform,
            _camera_buffer,
            camera_bind_group,
            cube,
            cube_trackball,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.configure(&self.config);
            self.camera_uniform.update_view_proj(&self.camera);
            self.queue.write_buffer(
                &self._camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
            self.depth_texture = texture::Texture::create_depth_texture(
                "depth_texture",
                &self.device,
                &self.config,
                match WORLD_HANDEDNESS {
                    Hand::Left => wgpu::CompareFunction::LessEqual,
                    Hand::Right => wgpu::CompareFunction::GreaterEqual,
                },
            );
            self.cube_trackball.set_viewport_size(&new_size);
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.cube_trackball.handle_window_event(event)
    }

    pub fn update(&mut self) {

        let now = std::time::Instant::now();
        let cube_to_world = self.cube_trackball.orientation(now);
        self.cube.update_transform(&cube_to_world);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let z_far = match WORLD_HANDEDNESS {
            Hand::Left => 1.0,
            Hand::Right => 0.0,
        };
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            }
        );

        let cube_prepared_data = self.cube.prepare(
            &cube::CubeAttributes {}
        );
        
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: & self.depth_texture.view,
                            depth_ops: Some(
                                wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(z_far),
                                    store: true,
                                }
                            ),
                            stencil_ops: None,
                        }
                    ),
                },
            );

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            self.cube.render(
                &self.queue,
                &mut render_pass,
                &cube_prepared_data,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct Stats {
    frame_count: u32,
    prev_frame_count: u32,
    prev_time: std::time::Instant,
}

impl Stats {
    fn new() -> Self {
        Stats {
            frame_count: 0,
            prev_frame_count: 0,
            prev_time: std::time::Instant::now(),
        }
    }
    fn count_frame(&mut self) {

        self.frame_count += 1;

        let now = std::time::Instant::now();
        let dur = now.duration_since(self.prev_time);
                if dur.as_secs() > 0 {
            if self.prev_frame_count != 0 {
                let n = self.frame_count - self.prev_frame_count;
                println!(
                    "{0:.2} frames/second",
                    n as f64 / dur.as_secs_f64(),
                );
            }
            self.prev_time = now;
            self.prev_frame_count = self.frame_count;
            }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello WGPU")
        .build(&event_loop)
        .unwrap();
    let mut state = pollster::block_on(State::new(&window));
    let mut stats = Stats::new();

    event_loop.run(move |event, _, control_flow| {
        match event {

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.handle_window_event(event) {
                    match event {

                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,

                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }

                        WindowEvent::ScaleFactorChanged {
                            new_inner_size,
                            ..
                        } => {
                            state.resize(**new_inner_size);
                        }

                        _ => {}
                    }
                }
            },

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) =>
                        *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
                stats.count_frame();
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        }
    });
}
