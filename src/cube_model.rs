use std::io::BufReader;

use cgmath::{
    Deg,
    Matrix4,
    Vector3,
};
use stringreader::StringReader;

const FACE_LENGTH_MM: f32 = 128.0;
const FACE_DISPLACEMENT_MM: f32 = 3.6;

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
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EdgeVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],       // XXX
}

impl CubeModel {
    pub fn new() -> Self {

        let obj = Self::parse_obj();
        let (models, _materials) = obj.unwrap();
        // println!("materials = {:?}", materials);
        // println!("materials.1 = {:?}", materials.1);
        // println!("materials.2 = {:?}", materials.2);
        // let materials = materials.unwrap();

        // println!("number of models = {}", models.len());
        // println!("number of materials = {}", materials.len());
        // for (i, m) in materials.iter().enumerate() {
        //     println!("material[{}]:", i);
        //     println!("  {:?}", m);
        // }
        // for (i, m) in models.iter().enumerate() {
        //     let mesh = &m.mesh;
        //     println!("model[{}]:", i);
        //     println!("  name = \"{}\"", m.name);
        //     println!("  mesh:");
        //     println!("    positions = {:?}", mesh.positions.len());
        //     println!("    vertex_color = {:?}", mesh.vertex_color.len());
        //     println!("    normals = {:?}", mesh.normals.len());
        //     println!("    texcoords = {:?}", mesh.texcoords.len());
        //     println!(
        //         "    indices = {:?}, max {:?}",
        //         mesh.indices.len(),
        //         mesh.indices.iter().max(),
        //     );
        //     println!("    face_arities = {:?}", mesh.face_arities.len());
        //     println!(
        //         "    texcoord_indices = {:?}, max {:?}",
        //         mesh.texcoord_indices.len(),
        //         mesh.texcoord_indices.iter().max(),
        //     );
        //     println!(
        //         "    normal_indices = {:?}, max {:?}",
        //         mesh.normal_indices.len(),
        //         mesh.normal_indices.iter().max(),
        //     );
        //     println!("    mesh.material_id = {:?}", mesh.material_id);
        //     println!();
        // }
        // println!("{:?}", models[0].mesh.indices);
        // println!("{:?}", models[0].mesh.texcoord_indices);
        // println!("{:?}", models[0].mesh.normal_indices);
        assert!(models[0].mesh.indices == models[0].mesh.normal_indices);

        // fn pto<T>(_: &T) {
        //     println!("{}", std::any::type_name::<T>());
        // }

        // for i in std::iter::zip(
        //     models[0].mesh.positions.chunks(3),
        //     models[0].mesh.normals.chunks(3),
        // ).take(5) {
        //     println!("i = {:?}", i);
        // };

        // fn map_foo<T: std::fmt::Debug>(_arg: T) -> u32 {
        //     println!("map_foo: {:?}", _arg);
        //     0
        // }

        // let _x: Vec<u32> = std::iter::zip(
        //     models[0].mesh.positions.chunks(3),
        //     models[0].mesh.normals.chunks(3)
        // ).take(5).map(map_foo::<_>).collect::<Vec<u32>>();
        // println!("_x = {:?}", _x);
        // println!();

        // let edge_vertices = std::iter::zip(
        //     models[0].mesh.positions.chunks(3),
        //     models[0].mesh.normals.chunks(3)
        // ).take(2).map(
        //     |(pos, norm)| {
        //         // println!("pos = {:?}", pos);
        //         // // pto(&pos);
        //         // println!("pos.len() = {}", pos.len());
        //         EdgeVertex {
        //             position: [pos[0], pos[1], pos[2]],
        //             normal: [norm[0], norm[1], norm[2]],
        //         }
        //     }
        // ).collect::<Vec<EdgeVertex>>();
        // println!("edge_vertices = {:?}", edge_vertices);

        let edge_vertices: Vec<EdgeVertex> = std::iter::zip(
            models[0].mesh.positions.chunks(3),
            models[0].mesh.normals.chunks(3)
        ).map(
            |(pos, norm)| {
                // println!("pos = {:?}", pos);
                // // pto(&pos);
                // println!("pos.len() = {}", pos.len());
                EdgeVertex {
                    position: [pos[0], pos[1], pos[2]],
                    normal: [norm[0], norm[1], norm[2]],
                    tex_coords: [0.5, 0.5],
                }
            }
        ).collect();

        let edge_tri_count: usize = models[0].mesh.indices.len() / 3;
        let mut _foo: Vec<EdgeVertex> = vec![Default::default(); edge_tri_count];
        let mut _bar: Vec<u32> = vec![0; edge_tri_count];

        let mut out = Self {
            face_vertices: Vec::new(),
            face_indices: Vec::new(),
            face_xforms: Vec::new(),
            edge_vertices: Vec::new(),
            edge_indices: Vec::new(),
        };

        const HFL: f32 = FACE_LENGTH_MM / 2.0;  // half face length

        out.face_vertices.push(
            FaceVertex {                // upper left
                position: [-HFL, HFL, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
        );
        out.face_vertices.push(
            FaceVertex {                // upper right
                position: [HFL, HFL, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
        );
        out.face_vertices.push(
            FaceVertex {                // lower left
                position: [-HFL, -HFL, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
        );
        out.face_vertices.push(
            FaceVertex {                // lower right
                position: [HFL, -HFL, 0.0],
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
        let tran = Matrix4::<f32>::from_translation(
            (HFL + FACE_DISPLACEMENT_MM) * z
        );

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

        out.edge_vertices = edge_vertices;
        out.edge_indices = models[0].mesh.indices.clone();

        out
    }

    fn parse_obj() -> tobj::LoadResult {

        let obj_source = include_str!("filleted_cube.obj");
        let string_reader = StringReader::new(obj_source);
        let mut buf_reader = BufReader::new(string_reader);

        tobj::load_obj_buf(
            &mut buf_reader,
            &tobj::LoadOptions { ..Default::default() },
            Self::null_material_loader,
        )
    }

    // fn _path_material_loader(_p: &std::path::Path) -> tobj::MTLLoadResult {
    //     println!("null material load {:?}", _p);
    //     let path = std::path::Path::new("src").join(_p);
    //     println!("path = {:?}", path);
    //     tobj::load_mtl(&path)
    // }

    fn null_material_loader(_p: &std::path::Path) -> tobj::MTLLoadResult {
        let mut materials = Vec::<tobj::Material>::new();
        let mut index = ahash::AHashMap::<std::string::String, usize>::new();
        for (i, name) in [
            "Paint_-_Enamel_Glossy_(Black)",
            "Steel_-_Satin",
        ].iter().enumerate() {
            let name: std::string::String = name.to_string();
            materials.push(
                tobj::Material {
                    name: name.clone(),
                    ..Default::default()
                }
            );
            index.insert(name, i);
        }
        Ok((materials, index))
    }
}
