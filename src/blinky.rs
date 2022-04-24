use crate::test_pattern;
use crate::traits::Renderable;

pub struct Blinky {
    test_pattern: test_pattern::TestPattern,
    blinky_texture: wgpu::Texture,
    blinky_texture_view: wgpu::TextureView,
}

impl Blinky {
    pub fn new(device: &wgpu::Device) -> Self {
        let test_pattern = test_pattern::TestPattern::new();
        let blinky_texture = device.create_texture(&wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
        });
        let blinky_texture_view =
            blinky_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("blinky_texture_view"),
                ..Default::default()
            });

        Self {
            test_pattern,
            blinky_texture,
            blinky_texture_view,
        }
    }

    pub fn blinky_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.blinky_texture_view)
    }

    pub fn update(&mut self) {
        self.test_pattern.next_frame();
    }
}

pub struct BlinkyAttributes();
pub struct BlinkyPreparedData {}

impl Renderable<BlinkyAttributes, BlinkyPreparedData> for Blinky {
    fn prepare(&self, _: &BlinkyAttributes) -> BlinkyPreparedData {
        BlinkyPreparedData {}
    }

    fn render(
        &self,
        queue: &wgpu::Queue,
        _: &mut wgpu::RenderPass,
        _: &BlinkyPreparedData,
    ) {
        queue.write_texture(
            self.blinky_texture.as_image_copy(),
            self.test_pattern.current_frame(),
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
    }
}
