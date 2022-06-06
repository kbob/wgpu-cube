// prefloor
//  create texture
//  create render pass
// texture is in screen space
//  resolution is 256 in vertical dimension
//  aspect matches viewport aspect
//
// render glow into texture

const GLOW_HEIGHT: u32 = 256;

#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub width: u32,
    pub height: u32,
}

pub struct PreFloor {
    glow_view: wgpu::TextureView,
    glow_sampler: wgpu::Sampler,
    pipeline: wgpu::RenderPipeline,
}

impl PreFloor {
    pub fn new(
        device: &wgpu::Device,
        config: &Configuration,
        shader: &wgpu::ShaderModule,
        static_binding_layout: &wgpu::BindGroupLayout,
        frame_binding_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let (glow_view, glow_sampler) = Self::create_glow(device, config);
        let pipeline = {
            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("prefloor_pipeline_layout"),
                    bind_group_layouts: &[
                        static_binding_layout,
                        frame_binding_layout,
                    ],
                    push_constant_ranges: &[],
                },
            );
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("prefloor_pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_floor_main",
                    buffers: &[crate::floor::FloorVertexRaw::desc()],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // fragment: None,
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: "fs_prefloor_main",
                    targets: &[
                        wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba16Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }
                    ],
                }),
                multiview: None,
            })
        };
        Self {
            glow_view,
            glow_sampler,
            pipeline,
        }
    }

    // XXX need to recreate forward pass bind group.
    // It works well enough with the default aspect ratio...
    // pub fn resize(&mut self, device: &wgpu::Device, config: &Configuration) {
    //     let (glow_view, glow_sampler) = Self::create_glow(device, config);
    //     self.glow_view = glow_view;
    //     self.glow_sampler = glow_sampler;
    // }

    pub fn update(&mut self) {}

    pub fn glow_view_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.glow_view)
    }

    pub fn glow_sampler_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.glow_sampler)
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        other_bind_groups: &[&wgpu::BindGroup],
        vertex_slice: wgpu::BufferSlice,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("prefloor_render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.glow_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        render_pass.set_pipeline(&self.pipeline);
        for (i, bg) in other_bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bg, &[]);
        }
        render_pass.set_vertex_buffer(0, vertex_slice);
        render_pass.draw(0..6, 0..1);
    }

    fn create_glow(
        device: &wgpu::Device,
        config: &Configuration,
    ) -> (wgpu::TextureView, wgpu::Sampler) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("prefloor_glow_texture"),
            size: wgpu::Extent3d {
                width: GLOW_HEIGHT * config.width / config.height,
                height: GLOW_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("prefloor_glow_view"),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("prefloor_glow_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        (view, sampler)
    }
}
