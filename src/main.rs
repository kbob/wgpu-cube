use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

mod cube_model;
mod texture;
mod trackball;
use trackball::{
    Manipulable,
    Responder,
};

// Question:
//    Should each instance be a face and the cube be uniform?
//    or should face data be per-vertex and the cube be an instance?
//  A is slightly more efficient?
//  B is more general, if I ever want two cubes.
//
//  Choose B.
//      The cube model has six faces, each transformed to cube coordinates.

// Transformation sequence.
//
// face coords
//                                  precomputed in cube model
// cube coords
//                                  instance.cube_to_world
// world coords
//                                  uniform.world_to_view
// camera/eye/view coords
//                                  uniform.view_to_NDC
// Normalized Device Coordinates
//                                  magic in wgpu
// framebuffer coords

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0,  1.0,  0.0,  0.0,
    0.0,  0.0, -0.5,  0.0,
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
        return OPENGL_TO_WGPU_MATRIX * proj * view;
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

struct FaceInstance {
    cube_to_world: cgmath::Matrix4<f32>,
    face_to_cube: cgmath::Matrix4<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FaceInstanceRaw {
    cube_to_world: [[f32; 4]; 4],
    face_to_cube: [[f32; 4]; 4],
}

impl FaceInstance {
    fn to_raw(&self) -> FaceInstanceRaw {
        FaceInstanceRaw {
            cube_to_world: self.cube_to_world.into(),
            face_to_cube: self.face_to_cube.into(),
        }
    }
    fn update_cube_xform(&mut self, new_xform: &cgmath::Matrix4<f32>) {
        self.cube_to_world = *new_xform;
    }
}

impl FaceInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<FaceInstanceRaw>(
                ) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },

                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 24]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 28]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
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
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format|
                wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    // depth_compare: wgpu::CompareFunction::Less,
                    depth_compare: wgpu::CompareFunction::Greater,
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
    cube_trackball: trackball::Trackball,
    cube_face_instances: Vec<FaceInstance>,
    cube_face_instance_data: Vec<FaceInstanceRaw>,
    cube_face_instance_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
    face_index_count: u32,
    face_vertex_buffer: wgpu::Buffer,
    face_index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(window: &Window) -> Self {

        // Device and Surface

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        // for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
        //     println!("adapter = {:?}", adapter);
        //     println!("  info = {:?}", adapter.get_info());
        //     println!("  features = {:?}", adapter.features());
        //     println!("  limits = {:?}", adapter.limits());
        //     println!(
        //         "  downlevel caps = {:?}",
        //         adapter.get_downlevel_properties()
        //     );
        //     println!("  textures:");
        //     println!(
        //         "    Depth24Plus         = {:?}",
        //         adapter.get_texture_format_features(
        //             wgpu::TextureFormat::Depth24Plus
        //         )
        //     );
        //     println!(
        //         "    Depth24PlusStencil8 = {:?}",
        //         adapter.get_texture_format_features(
        //             wgpu::TextureFormat::Depth24PlusStencil8
        //         )
        //     );
        //     println!(
        //         "    Depth32Float        = {:?}",
        //         adapter.get_texture_format_features(
        //             wgpu::TextureFormat::Depth32Float
        //         )
        //     );
        // }
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: Some("device"),
            },
            None,
        ).await.unwrap();

        // println!("window size = {:?}", size);
        // println!("window scale factor = {:?}", window.scale_factor());
        // println!(
        //     "surface preferred texture format = {:?}",
        //     surface.get_preferred_format(&adapter)
        // );
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
            eye: (1.0, 1.0, 2.0).into(),
            // eye: (0.0, 0.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
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
                label: Some("camera_bind_group_layout"),
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: _camera_buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            }
        );

        // Cube Model

        let model = cube_model::CubeModel::new();

        let cube_trackball = trackball::Trackball::new(&size);

        let cube_face_instances = model.face_xforms.iter().map( {
            |xform|
            FaceInstance {
                cube_to_world: cgmath::Matrix4::identity(),
                face_to_cube: *xform,
            }
        }).collect::<Vec<FaceInstance>>();

        let shader_text = include_str!("cube_face_shader.wgsl");
        let cube_face_shader = wgpu::ShaderModuleDescriptor {
            label: Some("cube_face_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_text.into()),
        };

        let face_index_count = model.face_indices.len() as u32;

        let face_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("face_vertex_buffer"),
                contents: bytemuck::cast_slice(model.face_vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let face_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("face_index_buffer"),
                contents: bytemuck::cast_slice(model.face_indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let cube_face_instance_data = cube_face_instances.iter().map(
            FaceInstance::to_raw
        ).collect::<Vec<_>>();

        let cube_face_instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("cube_face_instance_buffer"),
                contents: bytemuck::cast_slice(&cube_face_instance_data),
                usage: wgpu::BufferUsages::VERTEX |
                       wgpu::BufferUsages::COPY_DST,
            }
        );

        // Cube Face Texture

        let diffuse_bytes = include_bytes!("hi.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            diffuse_bytes,
            "hi.png"
        ).unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
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
                label: Some("texture_bind_group_layout"),
            }
        );

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &diffuse_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &diffuse_texture.sampler,
                        ),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        // Pipeline

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            &config,
            "depth_texture",
        );
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline_layout (cube faces)"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );
        let render_pipeline = create_render_pipeline(
            "cube_face_pipeline",       // label
            &device,                    // device
            &pipeline_layout,           // layout
            config.format,              // color_format
            Some(texture::Texture::DEPTH_FORMAT), // depth_format
            &[                          // vertex_layouts
                cube_model::FaceVertex::desc(),
                FaceInstanceRaw::desc(),
            ],
            cube_face_shader,           // shader
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
            cube_trackball,
            cube_face_instances,
            cube_face_instance_data,
            cube_face_instance_buffer,
            diffuse_bind_group,
            face_index_count,
            face_vertex_buffer,
            face_index_buffer,
            render_pipeline,
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
                &self.device,
                &self.config, "depth_texture",
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
        for fi in &mut self.cube_face_instances {
            fi.update_cube_xform(&cube_to_world);
        }
        for fir in &mut self.cube_face_instance_data {
            fir.cube_to_world = cube_to_world.into();
        }
        self.queue.write_buffer(
            &self.cube_face_instance_buffer,
            0,
            bytemuck::cast_slice(self.cube_face_instance_data.as_slice()),
        );
        
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.00250,
                                        g: 0.00625,
                                        b: 0.01500,
                                        a: 1.0,
                                    }
                                ),
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: & self.depth_texture.view,
                            depth_ops: Some(
                                wgpu::Operations {
                                    // load: wgpu::LoadOp::Clear(1.0),
                                    load: wgpu::LoadOp::Clear(0.0),
                                    store: true,
                                }
                            ),
                            stencil_ops: None,
                        }
                    ),
                },
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.face_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(
                1,
                self.cube_face_instance_buffer.slice(..),
            );
            render_pass.set_index_buffer(
                self.face_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32
            );
            let face_instance_count = self.cube_face_instances.len();
            render_pass.draw_indexed(
                0..self.face_index_count,
                0,
                0..face_instance_count as _
                );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    // let window = WindowBuilder::new().build(&event_loop).unwrap();
    // let a = winit::dpi::PhysicalSize::<u32> { width: 200, height: 400 };
    // let b = winit::dpi::Size::Physical(a);
    let window = WindowBuilder::new()
        .with_title("Hello WGPU")
        // .with_inner_size(b)
        .build(&event_loop)
        .unwrap();
    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        // println!("event = {:?}", event);
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
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }
            
            _ => {}
        }
    });
}
