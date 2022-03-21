use cgmath::{
    Basis3,
    Point3,
    Quaternion,
    Vector3,
};
use cgmath::prelude::*;

pub const SIDE: f32 = 128.0;
pub const EDGE_WIDTH: f32 = 3.0;
pub const FACES: [Quaternion<f32>; 6] = [
    Quaternion::from_sv(0.0, Vector3::new(-1.0, 0.0, 0.0)),
    Quaternion::from_sv(0.0, Vector3::new(0.0, 1.0, 0.0)),
    Quaternion::from_sv(0.0, Vector3::new(1.0, 0.0, 0.0)),
    Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, -1.0)),
    Quaternion::from_sv(0.0, Vector3::new(0.0, -1.0, 0.0)),
    Quaternion::from_sv(0.0, Vector3::new(0.0, 0.0, 1.0)),
];

#[derive(Debug)]
pub struct CubeModel {
    pub face_vertices: Vec<FaceVertex>,
    pub face_indices: Vec<u32>,
    pub edge_vertices: Vec<EdgeVertex>,
    pub edge_indices: Vec<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FaceVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl FaceVertex {
    // pub fn desc0<'a>() -> wgpu::VertexBufferLayout<'a> {
    //     wgpu::VertexBufferLayout {
    //         array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
    //         step_mode: wgpu::VertexStepMode::Vertex,
    //         attributes: &[
    //             wgpu::VertexAttribute {
    //                 offset: 0,
    //                 shader_location: 0,
    //                 format: wgpu::VertexFormat::Float32x3,
    //             },
    //             wgpu::VertexAttribute {
    //                 offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
    //                 shader_location: 1,
    //                 format: wgpu::VertexFormat::Float32x3,
    //             },
    //             wgpu::VertexAttribute {
    //                 offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
    //                 shader_location: 1,
    //                 format: wgpu::VertexFormat::Float32x2,
    //             },
    //         ],
    //     }
    // }

    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2
        ];
    
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<FaceVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EdgeVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl CubeModel {
    pub fn new() -> Self {
        let mut out = Self {
            face_vertices: Vec::new(),
            face_indices: Vec::new(),
            edge_vertices: Vec::new(),
            edge_indices: Vec::new(),
        };

        for face in FACES {
            // println!("face = {:?}", face);
            let rot = Basis3::from_quaternion(&face);
            let half = SIDE / 2.0;
            let haew = half + EDGE_WIDTH / 2.0;
            let _normal = rot.rotate_vector(Vector3::unit_z());
            for u in [-half, half] {
                for v in [-half, half] {
                    let corner = Point3::<f32>::new(u, v, haew);
                    let _corner = rot.rotate_point(corner);
                    // out.edge_vertices.push(
                    //     CubeVertex {
                    //         position: corner.into(),
                    //         normal: normal.into(),
                    //     }
                    // );
                }
            }
        }
        out.face_vertices.push(
            FaceVertex {                // upper left
                position: [-0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
        );
        out.face_vertices.push(
            FaceVertex {                // upper right
                position: [0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
        );
        out.face_vertices.push(
            FaceVertex {                // lower left
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
        );
        out.face_vertices.push(
            FaceVertex {                // lower right
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
        );

        out.face_indices.push(0);
        out.face_indices.push(2);
        out.face_indices.push(1);

        out.face_indices.push(1);
        out.face_indices.push(2);
        out.face_indices.push(3);

        out
    }
}
