use cgmath::{
    Deg,
    Matrix4,
    Vector3,
};

#[derive(Debug)]
pub struct CubeModel {
    pub face_vertices: Vec<FaceVertex>,
    pub face_indices: Vec<u32>,
    pub face_xforms: Vec<Matrix4<f32>>,

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
            face_xforms: Vec::new(),
            edge_vertices: Vec::new(),
            edge_indices: Vec::new(),
        };

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

        let z = Vector3::<f32>::unit_z();
        let mut tran = Matrix4::<f32>::from_translation(0.5 * z);
        tran = tran * Matrix4::<f32>::from_translation(3.0 / 128.0 * 0.5 * z);

        {
            let rot1 = Matrix4::from_angle_z(Deg::<f32>(180.0));
            let rot2 = Matrix4::from_angle_y(Deg::<f32>(-90.0));
            out.face_xforms.push(rot2 * rot1 * tran);   // 1: left
        }
        {
            let rot1 = Matrix4::from_angle_z(Deg::<f32>(180.0));
            out.face_xforms.push(rot1 * tran);          // 2: front
        }
        {
            let rot1 = Matrix4::from_angle_z(Deg::<f32>(180.0));
            let rot2 = Matrix4::from_angle_y(Deg::<f32>(90.0));
            out.face_xforms.push(rot2 * rot1 * tran);   // 3: right
        }
        {
            let rot1 = Matrix4::from_angle_z(Deg::<f32>(90.0));
            let rot2 = Matrix4::from_angle_x(Deg::<f32>(90.0));
            out.face_xforms.push(rot2 * rot1 * tran);   // 4: bottom
        }
        {
            let rot1 = Matrix4::from_angle_z(Deg::<f32>(90.0));
            let rot2 = Matrix4::from_angle_x(Deg::<f32>(180.0));
            out.face_xforms.push(rot2 * rot1 * tran);   // 5: back
        }
        {
            let rot1 = Matrix4::from_angle_z(Deg::<f32>(90.0));
            let rot2 = Matrix4::from_angle_x(Deg::<f32>(-90.0));
            out.face_xforms.push(rot2 * rot1 * tran);   // 6: top
        }

        out
    }
}
