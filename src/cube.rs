use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::cube_model;
use crate::texture;
use crate::traits::Renderable;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CubeUniformRaw {
    cube_to_world: [[f32; 4]; 4],
    decal_is_visible: u32,
    _padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FaceStaticInstanceRaw {
    face_to_cube: [[f32; 4]; 4],
    decal_offset: [f32; 2],
}

impl FaceStaticInstanceRaw {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] =
    wgpu::vertex_attr_array![
        5 => Float32x4,         // face_to_cube: mat4<f32>
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
        9 => Float32x2          // decal_offset: vec2<f32>
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Cube {

    // Whole Cube Data

    cube_to_world: cgmath::Matrix4<f32>,
    cube_uniform_buffer: wgpu::Buffer,
    cube_uniform_bind_group: wgpu::BindGroup,

    // Face Data

    face_instance_count: u32,
    face_instance_buffer: wgpu::Buffer,
    face_vertex_buffer: wgpu::Buffer,
    face_vertex_index_count: u32,
    face_vertex_index_buffer: wgpu::Buffer,
    face_decal_bind_group: wgpu::BindGroup,
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
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {

        // create static data here:
        //      cube_to_world transform
        //      cube uniform buffer
        //      cube uniform bind group
        //
        //      face static instance buffer
        //          references instance data
        //      face vertex buffer
        //          references vertex data
        //      face vertex index buffer
        //          references vertex index data
        //      face decal bind group
        //          references texture view, sampler
        //          !!! DOES NOT REFERENCE DATA !!!
        //      face pipeline
        //          references shader, bind group layouts, vertex formats
        //
        //      edge vertex buffer
        //          references vertex data
        //      edge vertex index buffer
        //          references vertex index data
        //      edge pipeline
        //          references shader, bind group layouts, vertex formats


        let model = cube_model::CubeModel::new();

        let cube_to_world = cgmath::Matrix4::identity();

        // N.B., the cube uniform buffer is not initialized.
        // It will be updated before the first render.
        let cube_uniform_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("cube_uniform_buffer"),
                size: std::mem::size_of::<CubeUniformRaw>() as u64,
                usage: (wgpu::BufferUsages::UNIFORM |
                        wgpu::BufferUsages::COPY_DST),
                mapped_at_creation: false,
            }
        );

        let cube_uniform_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("cube_bind_group"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: (
                            wgpu::ShaderStages::VERTEX |
                            wgpu::ShaderStages::FRAGMENT
                        ),
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
                    decal_offset: [
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
        let face_decal_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("face_decal_bind_group_layout"),
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
        let face_decal_bind_group = {
            let decal_bytes = include_bytes!("DIN_digits_linear.png");
            let decal_texture = texture::Texture::from_bytes(
                &device,
                &queue,
                decal_bytes,
                "DIN_digits_linear.png",
            ).unwrap();
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &face_decal_bind_group_layout,
                    label: Some("face_decal_bind_group"),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &decal_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &decal_texture.sampler,
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
                        &face_decal_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                }
            );
            let shader_text = include_str!("cube_face_shader.wgsl");
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
            cube_to_world,
            cube_uniform_buffer,
            cube_uniform_bind_group,

            face_instance_count,
            face_instance_buffer,
            face_vertex_buffer,
            face_vertex_index_count,
            face_vertex_index_buffer,
            face_decal_bind_group,
            face_pipeline,

            edge_vertex_buffer,
            edge_vertex_index_count,
            edge_vertex_index_buffer,
            edge_pipeline,
        }
    }

    pub fn update_transform(&mut self, xform: &cgmath::Matrix4<f32>)
    {
        self.cube_to_world = *xform;
    }
}

pub struct CubePreparedData {
    // store any data that is submitted per-frame here.
    cube_uniform: CubeUniformRaw,
    // is this where the video goes?
}

pub struct CubeAttributes {}

impl Renderable<CubeAttributes, CubePreparedData> for Cube {

    fn prepare(&self, _attr: &CubeAttributes) -> CubePreparedData
    {
        CubePreparedData {
            cube_uniform: CubeUniformRaw {
                cube_to_world: self.cube_to_world.into(),
                decal_is_visible: true as u32,
                _padding: [0, 0, 0],
            },
        }
    }

    fn render<'rpass>(
        &'rpass self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        prepared: &'rpass CubePreparedData,
    ) {
        // Transmit transform.

        queue.write_buffer(
            &self.cube_uniform_buffer,
            0,
            bytemuck::cast_slice(&[prepared.cube_uniform]),
        );

        // Render Faces

        render_pass.set_pipeline(&self.face_pipeline);
        // Camera bind group is set elsewhere.
        // render_pass.set_bind_group(0, &camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.cube_uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.face_decal_bind_group, &[]);
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
        // Camera bind group is set elsewhere.
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
