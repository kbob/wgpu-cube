use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

mod camera;
mod cube;
mod cube_model;
mod test_pattern;
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
pub enum Hand {
    Left,
    Right,
}
const WORLD_HANDEDNESS: Hand = Hand::Right;

const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.00250, g: 0.00625, b: 0.01500, a: 1.0,
};

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
    camera: camera::Camera,     // Buffalo buffalo Buffalo...
    cube: cube::Cube,           // ... buffalo buffalo buffalo...
    cube_trackball: trackball::Trackball,
    test_pattern: test_pattern::TestPattern,
    blinky_texture: wgpu::Texture,
    blinky_bind_group: wgpu::BindGroup,
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

        let camera = camera::Camera::new(
            &device,
            size.width,
            size.height,
            WORLD_HANDEDNESS,
        );

        // Blinky

        let test_pattern = test_pattern::TestPattern::new();
        let blinky_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("blinky_texture"),
                size: wgpu::Extent3d {
                    width: 6 * 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Uint,
                usage: (
                    wgpu::TextureUsages::TEXTURE_BINDING |
                    wgpu::TextureUsages::COPY_DST
                )
            }
        );
        let blinky_texture_view = blinky_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        let blinky_texture_sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: Some("blinky_sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        let blinky_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("blinky_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            }
        );
        let blinky_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("blinky_bind_group"),
                layout: &blinky_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &blinky_texture_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &blinky_texture_sampler,
                        ),
                    }
                ],
            }
        );

        // Cube Object

        let cube = cube::Cube::_new(
            &device,
            &queue,
            config.format,
            &camera.get_bind_group_layout(),
            &blinky_bind_group_layout,
        );

        let cube_trackball = trackball::Trackball::new(&size);

        // Depth Texture

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
            cube,
            cube_trackball,
            test_pattern,
            blinky_texture,
            blinky_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.set_aspect(self.config.width, self.config.height);
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
        let camera_prepared_data = self.camera.prepare(
            &camera::CameraAttributes {}
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
                            view: &self.depth_texture.view,
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

            self.queue.write_texture(
                self.blinky_texture.as_image_copy(),
                self.test_pattern.next_frame(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(6 * 64 * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: 6 * 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
            );

            // Bind Groups
            //  0.  Camera Uniform
            //  1.  Blinky Texture
            //  2.  Cube Uniform
            //  3.  Cube Decal Texture
            self.camera.render(
                &self.queue,
                &mut render_pass,
                &camera_prepared_data,
            );
            render_pass.set_bind_group(1, &self.blinky_bind_group, &[]);
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
        if dur.as_secs() >= 1 {
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
    let ph = winit::dpi::PhysicalSize::new(1920, 1080);
    let window = WindowBuilder::new()
        .with_title("Hello WGPU")
        .with_inner_size(winit::dpi::Size::Physical(ph))
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
