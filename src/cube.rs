use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::cube_model;
use crate::texture;
use crate::traits::Renderable;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CubeUniformRaw {
    cube_to_world: [[f32; 4]; 4],
    decal_visibility: f32,
    _padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FaceStaticInstanceRaw {
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

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        let stride = std::mem::size_of::<Self>();
        assert!(stride % wgpu::VERTEX_STRIDE_ALIGNMENT as usize == 0);
        wgpu::VertexBufferLayout {
            array_stride: stride as _,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Cube {

    // Whole Cube Data

    cube_to_world: cgmath::Matrix4<f32>,
    cube_uniform_buffer: wgpu::Buffer,

    // Face Data

    face_instance_count: u32,
    face_instance_buffer: wgpu::Buffer,
    face_vertex_buffer: wgpu::Buffer,
    face_vertex_index_count: u32,
    face_vertex_index_buffer: wgpu::Buffer,
    face_decal_texture: texture::Texture,

    // Edge Data

    edge_vertex_buffer: wgpu::Buffer,
    edge_vertex_index_count: u32,
    edge_vertex_index_buffer: wgpu::Buffer,
}

impl Cube {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {

        // create static data here:
        //      cube_to_world transform
        //      cube uniform buffer
        //
        //      face static instance buffer
        //          references instance data
        //      face vertex buffer
        //          references vertex data
        //      face vertex index buffer
        //          references vertex index data
        //      face decal texture
        //          has texture, view, sampler.
        //
        //      edge vertex buffer
        //          references vertex data
        //      edge vertex index buffer
        //          references vertex index data

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

        let face_decal_texture = {
            let decal_bytes = include_bytes!("DIN_digits_aliased.png");
            texture::Texture::from_bytes(
                &device,
                &queue,
                decal_bytes,
                "DIN_digits_linear.png",
            ).unwrap()
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
        let edge_vertex_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("edge_vertex_index_buffer"),
                contents: bytemuck::cast_slice(model.edge_indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Self {
            cube_to_world,
            cube_uniform_buffer,

            face_instance_count,
            face_instance_buffer,
            face_vertex_buffer,
            face_vertex_index_count,
            face_vertex_index_buffer,
            face_decal_texture,

            edge_vertex_buffer,
            edge_vertex_index_count,
            edge_vertex_index_buffer,
        }
    }

    pub fn uniform_resource(&self) -> wgpu::BindingResource {
        self.cube_uniform_buffer.as_entire_binding()
    }

    pub fn face_decal_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.face_decal_texture.view)
    }

    pub fn update_transform(&mut self, xform: &cgmath::Matrix4<f32>)
    {
        self.cube_to_world = *xform;
    }
}

pub struct CubeFacePreparedData {
    cube_uniform: CubeUniformRaw,
}

pub struct CubeFaceAttributes {
    pub frame_time: f32,
}

impl Renderable<CubeFaceAttributes, CubeFacePreparedData> for Cube {

    fn prepare(&self, attr: &CubeFaceAttributes) -> CubeFacePreparedData
    {
        let phase = attr.frame_time as i32 % 2;
        let frac = attr.frame_time % 1.0;
        let brightness = if phase == 0 {
            frac * frac
        } else {
            (1.0 - frac) * (1.0 - frac)
        };
        CubeFacePreparedData {
            cube_uniform: CubeUniformRaw {
                cube_to_world: self.cube_to_world.into(),
                decal_visibility: brightness,
                _padding: [0, 0, 0],
            },
        }
    }

    fn render<'rpass>(
        &'rpass self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        prepared: &'rpass CubeFacePreparedData,
    ) {
        // Transmit transform.
        // XXX we transform before drawing faces, not before drawing edges.
        // XXX this relies on faces being drawn first.

        queue.write_buffer(
            &self.cube_uniform_buffer,
            0,
            bytemuck::cast_slice(&[prepared.cube_uniform]),
        );

        if false { return; }

        // Render Faces

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
    }
}

pub struct CubeEdgePreparedData();

pub struct CubeEdgeAttributes();

impl Renderable<CubeEdgeAttributes, CubeEdgePreparedData> for Cube {

    fn prepare(&self, _: &CubeEdgeAttributes) -> CubeEdgePreparedData {
        CubeEdgePreparedData {}
    }

    fn render<'rpass>(
        &'rpass self,
        _queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        _prepared: &'rpass CubeEdgePreparedData,
    ) {
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
