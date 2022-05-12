pub struct StaticBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl StaticBindings {
    pub const GROUP_INDEX: u32 = 0;
    const FACE_DECAL: u32 = 0;
    const CAMERA_UNIFORM: u32 = 1;
    const LIGHTS_UNIFORM: u32 = 2;
    const FLOOR_DECAL: u32 = 3;
    const FLOOR_DECAL_SAMPLER: u32 = 4;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("static_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::FACE_DECAL,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::CAMERA_UNIFORM,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
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
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::FLOOR_DECAL,
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
                        binding: Self::FLOOR_DECAL_SAMPLER,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
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
        floor_decal: wgpu::BindingResource,
        floor_decal_sampler: wgpu::BindingResource,
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
                wgpu::BindGroupEntry {
                    binding: Self::FLOOR_DECAL,
                    resource: floor_decal,
                },
                wgpu::BindGroupEntry {
                    binding: Self::FLOOR_DECAL_SAMPLER,
                    resource: floor_decal_sampler,
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
        blinky_texture: wgpu::BindingResource,
        cube_uniform: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::BLINKY_TEXTURE,
                    resource: blinky_texture,
                },
                wgpu::BindGroupEntry {
                    binding: Self::CUBE_UNIFORM,
                    resource: cube_uniform,
                },
            ],
        })
    }
}

pub struct ShadowPassBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl ShadowPassBindings {
    pub const GROUP_INDEX: u32 = 2;
    pub const SHADOW_UNIFORM: u32 = 0;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow_pass_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: Self::SHADOW_UNIFORM,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                }],
            });
        Self { layout }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        shadow_uniform: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_pass_bind_group"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: Self::SHADOW_UNIFORM,
                resource: shadow_uniform,
            }],
        })
    }
}

pub struct ForwardPassBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl ForwardPassBindings {
    pub const GROUP_INDEX: u32 = 2;
    pub const SHADOW_MAPS: u32 = 0;
    pub const SHADOW_MAPS_SAMPLER: u32 = 1;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("forward_pass_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::SHADOW_MAPS,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::SHADOW_MAPS_SAMPLER,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Comparison,
                        ),
                        count: None,
                    },
                ],
            });
        Self { layout }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        shadow_maps: wgpu::BindingResource,
        shadow_maps_sampler: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("forward_pass_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::SHADOW_MAPS,
                    resource: shadow_maps,
                },
                wgpu::BindGroupEntry {
                    binding: Self::SHADOW_MAPS_SAMPLER,
                    resource: shadow_maps_sampler,
                },
            ],
        })
    }
}

pub struct BlurPassBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl BlurPassBindings {
    pub const GROUP_INDEX: u32 = 2;
    pub const POST_UNIFORM: u32 = 0;
    pub const IMAGE_TEXTURE: u32 = 1;
    pub const IMAGE_SAMPLER: u32 = 2;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blur_pass_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::POST_UNIFORM,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::IMAGE_TEXTURE,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::IMAGE_SAMPLER,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        Self { layout }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        post_uniform: wgpu::BindingResource,
        image_texture: wgpu::BindingResource,
        image_sampler: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blur_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::POST_UNIFORM,
                    resource: post_uniform,
                },
                wgpu::BindGroupEntry {
                    binding: Self::IMAGE_TEXTURE,
                    resource: image_texture,
                },
                wgpu::BindGroupEntry {
                    binding: Self::IMAGE_SAMPLER,
                    resource: image_sampler,
                },
            ],
        })
    }
}

pub struct CompositePassBindings {
    pub layout: wgpu::BindGroupLayout,
}

impl CompositePassBindings {
    pub const GROUP_INDEX: u32 = 2;
    pub const POST_UNIFORM: u32 = 0;
    pub const COLOR_TEXTURE: u32 = 1;
    pub const COLOR_SAMPLER: u32 = 2;
    pub const BRIGHT_TEXTURE: u32 = 3;
    pub const BRIGHT_SAMPLER: u32 = 4;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blur_pass_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::POST_UNIFORM,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::COLOR_TEXTURE,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::COLOR_SAMPLER,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::BRIGHT_TEXTURE,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::BRIGHT_SAMPLER,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        Self { layout }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        post_uniform: wgpu::BindingResource,
        color_texture: wgpu::BindingResource,
        color_sampler: wgpu::BindingResource,
        bright_texture: wgpu::BindingResource,
        bright_sampler: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blur_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::POST_UNIFORM,
                    resource: post_uniform,
                },
                wgpu::BindGroupEntry {
                    binding: Self::COLOR_TEXTURE,
                    resource: color_texture,
                },
                wgpu::BindGroupEntry {
                    binding: Self::COLOR_SAMPLER,
                    resource: color_sampler,
                },
                wgpu::BindGroupEntry {
                    binding: Self::BRIGHT_TEXTURE,
                    resource: bright_texture,
                },
                wgpu::BindGroupEntry {
                    binding: Self::BRIGHT_SAMPLER,
                    resource: bright_sampler,
                },
            ],
        })
    }
}
