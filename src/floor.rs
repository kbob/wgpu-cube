use crate::texture;
use crate::traits::Renderable;
use wgpu::util::DeviceExt;

const FLOOR_HEIGHT: f32 = -120.0;
const FLOOR_WIDTH: f32 = 750.0;
const FLOOR_LENGTH: f32 = 750.0;

pub const FLOOR_BOUNDS_WORLD: cgmath::Ortho<f32> = cgmath::Ortho {
    left: 60.0 - FLOOR_WIDTH / 2.0,
    right: 60.0 + FLOOR_WIDTH / 2.0,
    bottom: FLOOR_HEIGHT,
    top: FLOOR_HEIGHT,
    near: 150.0,
    far: 150.0 - FLOOR_LENGTH,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FloorVertexRaw {
    position: [f32; 3],
    normal: [f32; 3],
    decal_coords: [f32; 2],
}

impl FloorVertexRaw {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        let stride = std::mem::size_of::<Self>();
        assert!(stride % wgpu::VERTEX_STRIDE_ALIGNMENT as usize == 0);
        wgpu::VertexBufferLayout {
            array_stride: stride as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Floor {
    pub decal: texture::Texture,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl Floor {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let decal_bytes = include_bytes!("grey-concrete-texture-2.png");
        let decal = texture::Texture::from_bytes(
            device,
            queue,
            decal_bytes,
            "floor_decal_texture",
        )
        .unwrap();
        let vertex_data = Self::create_vertex_data();
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("floor_vertex_buffer"),
                contents: bytemuck::cast_slice(vertex_data.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let vertex_count = vertex_data.len() as u32;
        Self {
            decal,
            vertex_buffer,
            vertex_count,
        }
    }

    pub fn decal_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.decal.view)
    }

    pub fn decal_sampler_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.decal.sampler)
    }

    pub fn vertex_slice(&self) -> wgpu::BufferSlice {
        self.vertex_buffer.slice(..)
    }

    fn create_vertex_data() -> Vec<FloorVertexRaw> {
        #[rustfmt::skip]
        let corners = [
            (0, 0), (1, 0), (0, 1), // NW triangle
            (1, 1), (0, 1), (1, 0), // SE triangle
        ];

        let mut data = Vec::new();
        for (i, j) in corners {
            let x = [FLOOR_BOUNDS_WORLD.left, FLOOR_BOUNDS_WORLD.right][i];
            let y = FLOOR_HEIGHT;
            let z = [FLOOR_BOUNDS_WORLD.near, FLOOR_BOUNDS_WORLD.far][j];
            let u = i as f32;
            let v = j as f32;
            data.push(FloorVertexRaw {
                position: [x, y, z],
                normal: [0.0, 1.0, 0.0],
                decal_coords: [u, v],
            })
        }

        data
    }
}

pub struct FloorAttributes();
pub struct FloorPreparedData {}

impl Renderable<FloorAttributes, FloorPreparedData> for Floor {
    fn prepare(&self, _: &FloorAttributes) -> FloorPreparedData {
        FloorPreparedData {}
    }

    fn render<'rpass>(
        &'rpass self,
        _: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        _: &'rpass FloorPreparedData,
    ) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
