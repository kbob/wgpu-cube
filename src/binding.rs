pub struct StaticBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl StaticBindings {
    pub const GROUP_INDEX: u32 = 0;
    const FACE_DECAL: u32 = 0;
    const CAMERA_UNIFORM: u32 = 1;
    const LIGHTS_UNIFORM: u32 = 2;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("static_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::FACE_DECAL,
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
                        binding: Self::CAMERA_UNIFORM,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::LIGHTS_UNIFORM,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        Self { layout }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        face_decal: wgpu::BindingResource,
        camera_uniform: wgpu::BindingResource,
        lights_uniform: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("static_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::FACE_DECAL,
                    resource: face_decal,
                },
                wgpu::BindGroupEntry {
                    binding: Self::CAMERA_UNIFORM,
                    resource: camera_uniform,
                },
                wgpu::BindGroupEntry {
                    binding: Self::LIGHTS_UNIFORM,
                    resource: lights_uniform,
                },
            ],
        })
    }
}

pub struct FrameBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl FrameBindings {
    pub const GROUP_INDEX: u32 = 1;
    const BLINKY_TEXTURE: u32 = 0;
    const CUBE_UNIFORM: u32 = 1;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("frame_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::BLINKY_TEXTURE,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::CUBE_UNIFORM,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        Self { layout }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        blinky: wgpu::BindingResource,
        cube_uniform: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::BLINKY_TEXTURE,
                    resource: blinky,
                },
                wgpu::BindGroupEntry {
                    binding: Self::CUBE_UNIFORM,
                    resource: cube_uniform,
                },
            ],
        })
    }
}
