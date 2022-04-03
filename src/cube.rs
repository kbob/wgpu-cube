// Good thoughts:
//      https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work
//
// Bevy's render stages:
//      Extract
//      Prepare
//      Queue
//      PhaseSort
//      Render
//      Cleanup
// https://docs.rs/bevy/latest/bevy/render/enum.RenderStage.html

// Scopes: uniform, instance, vertex
// Lifetimes: static, frame, shader, ???
// Visibility: vertex, fragment, both

use wgpu::util::DeviceExt;

use crate::cube_model;
use crate::texture;

pub trait Renderable<Attributes, PreparedData> {

    fn update(&mut self) {}     // optional method

    fn prepare(&self, _: &Attributes) -> PreparedData;

    fn render<'rpass>(
        &'rpass self,
        _: &mut wgpu::RenderPass<'rpass>,
        _: &'rpass PreparedData,
    );
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CubeUniformRaw {
    cube_to_world: [[f32; 4]; 4],
    decal_is_visible: u32,
}

impl CubeUniformRaw {
    fn new() -> Self {
        Self {
            cube_to_world: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            decal_is_visible: true as u32,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FaceStaticInstanceRaw {
    face_to_cube: [[f32; 4]; 4],
    texture_offset: [f32; 2],
}

impl FaceStaticInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // face_to_cube: 4 x 4 floats
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },

                // texture_offsets: vec2<u32>
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct FaceDynamicInstanceRaw {
//     cube_to_world: [[f32; 4]; 4],
// }

pub struct Cube {

    // Whole Cube Data

    #[allow(dead_code)]
    cube_uniform: CubeUniformRaw,
    #[allow(dead_code)]
    cube_uniform_buffer: wgpu::Buffer,
    // #[allow(dead_code)]
    cube_uniform_bind_group: wgpu::BindGroup,

    // Face Data

    face_instance_count: u32,
    face_instance_buffer: wgpu::Buffer,
    face_vertex_buffer: wgpu::Buffer,
    face_vertex_index_count: u32,
    face_vertex_index_buffer: wgpu::Buffer,
    face_texture_bind_group: wgpu::BindGroup,
    face_pipeline: wgpu::RenderPipeline,

    // Edge Data

    edge_vertex_buffer: wgpu::Buffer,
    edge_vertex_index_count: u32,
    edge_vertex_index_buffer: wgpu::Buffer,
    edge_pipeline: wgpu::RenderPipeline,
}

impl Cube {
    pub fn _new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
        // belt: &wgpu::util::StagingBelt,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // create static data here:
        //      face vertex data
        //      face vertex index data
        //      face instance static data
        //      face texture data
        //      face vertex buffer
        //      face vertex index buffer
        //      face static instance buffer
        //      face texture:
        //          texture
        //          buffer
        //          texture view
        //          bind group layout
        //          bind group
        //      face shader
        //      face pipeline layout
        //      face pipeline
        //
        //      edge vertex data
        //      edge vertex index data
        //      edge vertex buffer
        //      edge vertex index buffer
        //      edge shader
        //      edge pipeline layout
        //      edge pipeline

        let model = cube_model::CubeModel::new();
        let cube_uniform = CubeUniformRaw::new();
        let cube_uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("cube_uniform_buffer"),
                contents: bytemuck::cast_slice(&[cube_uniform]),
                usage: (
                    wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST
                ),
            }
        );
        let cube_uniform_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("cube_bind_group"),
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
                ]
            }
        );
        let cube_uniform_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("cube_bind_group"),
                layout: &cube_uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: cube_uniform_buffer.as_entire_binding(),
                    }
                ],
            }
        );

        // Face Initialization

        let face_instance_count = model.face_count;
        let face_instance_buffer = {
            let data = model.face_xforms.iter().enumerate().map( {
                |(i, xform)|
                FaceStaticInstanceRaw {
                    face_to_cube: (*xform as cgmath::Matrix4<f32>).into(),
                    texture_offset: [
                         (face_instance_count - i as u32 - 1) as f32,
                        0.0],
                }
            }).collect::<Vec<FaceStaticInstanceRaw>>();
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("face_instance_buffer"),
                    contents: bytemuck::cast_slice(data.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX |
                        wgpu::BufferUsages::COPY_DST,
                }
            )
        };
        let face_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("face_vertex_buffer"),
                contents: bytemuck::cast_slice(model.face_vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let face_vertex_index_count = model.face_indices.len() as u32;
        let face_vertex_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("face_vertex_index_buffer"),
                contents: bytemuck::cast_slice(model.face_indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let face_texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("DIN_digits_texture_bind_group_layout"),
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
            }
        );
        let face_texture_bind_group = {
            let texture_bytes = include_bytes!("DIN_digits_linear.png");
            let texture = texture::Texture::from_bytes(
                &device,
                &queue,
                texture_bytes,
                "DIN_digits_linear.png",
            ).unwrap();
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &face_texture_bind_group_layout,
                    label: Some("DIN_digits_texture_bind_group"),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &texture.sampler,
                            ),
                        },
                    ],
                }
            )
        };

        let face_pipeline = {
            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("face_pipeline_layout"),
                    bind_group_layouts: &[
                        &camera_bind_group_layout,
                        &cube_uniform_bind_group_layout,
                        &face_texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                }
            );
            let shader_text = include_str!("cube_face_shader_NEW.wgsl");
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("cube_face_shader"),
                source: wgpu::ShaderSource::Wgsl(shader_text.into()),
            };
            crate::create_render_pipeline(
                "face_pipeline",                        // label
                device,                                 // device
                &layout,                                // layout
                color_format,                           // color_format
                Some(texture::Texture::DEPTH_FORMAT),   // depth_format
                &[                                      // vertex_layouts
                    cube_model::FaceVertex::desc(),
                    FaceStaticInstanceRaw::desc(),
                ],
                shader,                                 // shader
            )
        };

        // Edge Initialization

        let edge_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("edge_vertex_buffer"),
                contents: bytemuck::cast_slice(model.edge_vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let edge_vertex_index_count = model.edge_indices.len() as u32;
        println!("edge_vertex_index_count = {:?}", edge_vertex_index_count);
        let edge_vertex_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("edge_vertex_index_buffer"),
                contents: bytemuck::cast_slice(model.edge_indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let edge_pipeline = {
            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("edge_pipeline_layout"),
                    bind_group_layouts: &[
                        &camera_bind_group_layout,
                        &cube_uniform_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                }
            );
            let shader_text = include_str!("cube_edge_shader.wgsl");
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("cube_edge_shader"),
                source: wgpu::ShaderSource::Wgsl(shader_text.into()),
            };
            crate::create_render_pipeline(
                "edge_pipeline",                        // label
                device,                                 // device
                &layout,                                // layout
                color_format,                           // color_format
                Some(texture::Texture::DEPTH_FORMAT),   // depth_format
                &[                                      // vertex_layouts
                    cube_model::EdgeVertex::desc(),
                ],
                shader,                                 // shader
            )
        };

        Self {
            cube_uniform,
            cube_uniform_buffer,
            cube_uniform_bind_group,

            face_instance_count,
            face_instance_buffer,
            face_vertex_buffer,
            face_vertex_index_count,
            face_vertex_index_buffer,
            face_texture_bind_group,
            face_pipeline,

            edge_vertex_buffer,
            edge_vertex_index_count,
            edge_vertex_index_buffer,
            edge_pipeline,
        }
    }
}

pub struct CubePreparedData {
    // create dynamic data here:
    //      face instance dynamic data
    // is this where the video goes?
}

pub struct CubeAttributes {}

impl Renderable<CubeAttributes, CubePreparedData> for Cube {

    fn prepare(&self, _attr: &CubeAttributes) -> CubePreparedData
    {
        CubePreparedData {}
    }

    fn render<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        _prepared: &'rpass CubePreparedData,
    ) {
        // Render Faces

        render_pass.set_pipeline(&self.face_pipeline);
        // Camera bind group is set elsewhere.
        // render_pass.set_bind_group(0, &camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.cube_uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.face_texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.face_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.face_instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.face_vertex_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(
            0..self.face_vertex_index_count,
            0,
            0..self.face_instance_count,
        );

        // Render Edges

        render_pass.set_pipeline(&self.edge_pipeline);
        // render_pass.set_bind_group(0, &camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.edge_vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.edge_vertex_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(
            0..self.edge_vertex_index_count,
            0,
            0..1,
        );
    }
}


// Future Topics
//  reattach the trackball to the cube
//  FaceInstanceRaw::desc() should use ATTRIBUTES constant.
//  floor
//  mirror
//  shadow
//  push constants for cube_to_world transform
