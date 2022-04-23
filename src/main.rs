use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

mod binding;
mod blinky;
mod camera;
mod cube;
mod cube_model;
mod floor;
mod lights;
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
const ALPHA_BLENDING: bool = false;
const SAMPLE_COUNT: u32 = 4;

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
                count: SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(
                            match ALPHA_BLENDING {
                                true => wgpu::BlendState::ALPHA_BLENDING,
                                false => wgpu::BlendState::REPLACE,
                            }
                        ),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                ],
            }),
            multiview: None,
        }
    )
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    device
        .create_texture(
            &wgpu::TextureDescriptor {
                label: Some("multisampleed_frame_texture"),
                size: multisampled_texture_extent,
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            }
        )
        .create_view(&wgpu::TextureViewDescriptor::default())
}

struct State {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_texture: texture::Texture,
    multisampled_framebuffer: wgpu::TextureView,
    camera: camera::Camera,     // Buffalo buffalo Buffalo...
    lights: lights::Lights,     // ... buffalo buffalo buffalo...
    blinky: blinky::Blinky,     // ... Buffalo buffalo.
    cube: cube::Cube,           // Upstate bison upstate...
    cube_trackball: trackball::Trackball,
    floor: floor::Floor,        // ... bison baffle baffle...
    cube_face_pipeline: wgpu::RenderPipeline,
    cube_edge_pipeline: wgpu::RenderPipeline,
    floor_pipeline: wgpu::RenderPipeline,
    static_bind_group: wgpu::BindGroup,
    frame_bind_group: wgpu::BindGroup,
    first_frame_time: Option<std::time::Instant>,
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

        // ensure textures big enough for full screen MSAA.
        let device_limits = wgpu::Limits {
            ..wgpu::Limits::default().using_resolution(adapter.limits())
        };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                features: wgpu::Features::empty(),
                limits: device_limits,
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

        // Lights

        let lights = lights::Lights::new(&device);

        // Blinky

        let blinky = blinky::Blinky::new(&device);

        // Cube Object

        let cube = cube::Cube::new(&device, &queue);

        let cube_trackball = trackball::Trackball::new(&size);

        // Floor object

        let floor = floor::Floor::new(&device, &queue);

        // Depth Texture

        let depth_texture = texture::Texture::create_depth_texture(
            "depth_texture",
            &device,
            &config,
            match WORLD_HANDEDNESS {
                Hand::Left => wgpu::CompareFunction::LessEqual,
                Hand::Right => wgpu::CompareFunction::GreaterEqual,
            },
            SAMPLE_COUNT,
        );

        // Multisampled Framebuffer

        let multisampled_framebuffer =
            create_multisampled_framebuffer(&device, &config);

        let static_bindings = binding::StaticBindings::new(&device);
        let frame_bindings = binding::FrameBindings::new(&device);
        let static_bind_group = static_bindings.create_bind_group(
            &device,
            cube.face_decal_resource(),
            camera.uniform_resource(),
            lights.uniform_resource(),
            floor.decal_resource(),
            floor.decal_sampler_resource(),
        );
        let frame_bind_group = frame_bindings.create_bind_group(
            &device,
            blinky.blinky_resource(),
            cube.uniform_resource(),
        );

        let cube_face_pipeline = {
            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("cube_face_pipeline_layout"),
                    bind_group_layouts: &[
                        &static_bindings.layout,
                        &frame_bindings.layout,
                    ],
                    push_constant_ranges: &[],
                }
            );
            let shader_text = include_str!("cube_face_shader.wgsl");
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("cube_face_shader"),
                source: wgpu::ShaderSource::Wgsl(shader_text.into()),
            };
            create_render_pipeline(
                "cube_face_pipeline",                   // label
                &device,                                // device
                &layout,                                // layout
                config.format,                          // color_format
                Some(texture::Texture::DEPTH_FORMAT),   // depth_format
                &[                                      // vertex_layouts
                    cube_model::FaceVertex::desc(),
                    cube::FaceStaticInstanceRaw::desc(),
                ],
                shader,                                 // shader
            )
        };

        let cube_edge_pipeline = {
            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("cube_edge_pipeline_layout"),
                    bind_group_layouts: &[
                        &static_bindings.layout,
                        &frame_bindings.layout,
                    ],
                    push_constant_ranges: &[],
                }
            );
            let shader_text = include_str!("cube_edge_shader.wgsl");
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("cube_edge_shader"),
                source: wgpu::ShaderSource::Wgsl(shader_text.into()),
            };
            create_render_pipeline(
                "cube_edge_pipeline",                   // label
                &device,                                // device
                &layout,                                // layout
                config.format,                          // color_format
                Some(texture::Texture::DEPTH_FORMAT),   // depth_format
                &[                                      // vertex_layouts
                    cube_model::EdgeVertex::desc(),
                ],
                shader,                                 // shader
            )
        };

        let floor_pipeline = {
            let layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("floor_pipeline_layout"),
                    bind_group_layouts: &[
                        &static_bindings.layout,
                        &frame_bindings.layout,
                    ],
                    push_constant_ranges: &[],
                });
            let shader_text = include_str!("floor_shader.wgsl");
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("floor_shader"),
                source: wgpu::ShaderSource::Wgsl(shader_text.into()),
            };
            create_render_pipeline(
                "floor_pipeline",                       // label
                &device,                                // device
                &layout,                                // layout
                config.format,                          // color_format
                Some(texture::Texture::DEPTH_FORMAT),   // depth_format
                &[                                      // vertex_layouts
                    floor::FloorVertexRaw::desc(),
                ],
                shader,                                 // shader
            )
        };

        let first_frame_time = None;

        // Results

        Self {
            size,
            surface,
            device,
            queue,
            config,
            depth_texture,
            multisampled_framebuffer,
            camera,
            lights,
            blinky,
            cube,
            cube_trackball,
            floor,
            cube_face_pipeline,
            cube_edge_pipeline,
            floor_pipeline,
            static_bind_group,
            frame_bind_group,
            first_frame_time,
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
                SAMPLE_COUNT,
            );
            self.multisampled_framebuffer =
                create_multisampled_framebuffer(&self.device, &self.config);
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
        self.blinky.update();
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

        let camera_prepared_data = self.camera.prepare(
            &camera::CameraAttributes {}
        );
        let lights_prepared_data = self.lights.prepare(
            &lights::LightsAttributes {}
        );
        let blinky_prepared_data = self.blinky.prepare(
            &blinky::BlinkyAttributes {}
        );
        let cube_face_prepared_data = self.cube.prepare(
            &cube::CubeFaceAttributes {
                frame_time:
                    self.first_frame_time
                        .get_or_insert_with(|| std::time::Instant::now())
                        .elapsed()
                        .as_secs_f32(),
            }
        );
        let cube_edge_prepared_data = self.cube.prepare(
            &cube::CubeEdgeAttributes {},
        );
        let floor_prepared_data = self.floor.prepare(
            &floor::FloorAttributes {}
        );
        
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                            view: &self.multisampled_framebuffer,
                            resolve_target: Some(&view),
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


            // Bind Groups
            //  0.  [Face Decal, Camera Uniform]
            //  1.  [Blinky Texture, Cube Uniform]
            render_pass.set_bind_group(
                binding::StaticBindings::GROUP_INDEX,
                &self.static_bind_group,
                &[],
            );
            render_pass.set_bind_group(
                binding::FrameBindings::GROUP_INDEX,
                &self.frame_bind_group,
                &[],
            );

            if true {
                // LED animation
                self.blinky.render(
                    &self.queue,
                    &mut render_pass,
                    &blinky_prepared_data,
                );
            }
            self.camera.render(
                &self.queue,
                &mut render_pass,
                &camera_prepared_data,
            );
            self.lights.render(
                &self.queue,
                &mut render_pass,
                &lights_prepared_data,
            );
            if true {
                // cube faces
                render_pass.set_pipeline(&self.cube_face_pipeline);
                self.cube.render(
                    &self.queue,
                    &mut render_pass,
                    &cube_face_prepared_data,
                );
            }
            if true {
                // cube edges
                render_pass.set_pipeline(&self.cube_edge_pipeline);
                self.cube.render(
                    &self.queue,
                    &mut render_pass,
                    &cube_edge_prepared_data,
                );
            }
            if true {
                // floor
                render_pass.set_pipeline(&self.floor_pipeline);
                self.floor.render(
                    &self.queue,
                    &mut render_pass,
                    &floor_prepared_data,
                );
            }
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
    last_frame_time: std::time::Instant,
}

impl Stats {
    fn new() -> Self {
        let now = std::time::Instant::now();
        Stats {
            frame_count: 0,
            prev_frame_count: 0,
            prev_time: now,
            last_frame_time: now,
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
        self.last_frame_time = now;
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
                                    virtual_keycode:
                                        Some(VirtualKeyCode::Escape),
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
