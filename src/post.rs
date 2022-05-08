use crate::binding;
use wgpu::util::DeviceExt;

const BLUR_STEPS: u32 = 2;
const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniformRaw {
    blur_axis: u32,
    _padding: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PostVertexRaw {
    position: [f32; 4],
}

impl PostVertexRaw {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x4,
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

struct PostPass {
    render_pass_label: String,
    bind_group_index: u32,
    bind_group: wgpu::BindGroup,
}

pub struct Post {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    uniform_buffer: wgpu::Buffer,
    pub ldr_color: wgpu::TextureView,
    pub bright_color: wgpu::TextureView,
    ping: wgpu::TextureView,
    pong: wgpu::TextureView,
    blur_pass_bindings: binding::BlurPassBindings,
    composite_pass_bindings: binding::CompositePassBindings,
    hblur_pipeline: wgpu::RenderPipeline,
    vblur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    hblur0_pass: PostPass,
    hblur1_pass: PostPass,
    vblur_pass: PostPass,
    composite_pass: PostPass,
}

fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let data = PostUniformRaw {
        blur_axis: 0,
        _padding: [0, 0, 0],
    };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("post_uniform_buffer"),
        contents: bytemuck::cast_slice(&[data]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    })
}

fn create_vertex_buffer(device: &wgpu::Device) -> (wgpu::Buffer, u32) {
    #[rustfmt::skip]
    let corners = [
        (-1, -1), (1, -1), (-1, 1), // NW triangle
        (1, 1), (-1, 1), (1, -1), // SE triangle
    ];

    let mut data = Vec::new();
    for (i, j) in corners {
        let x = i as f32;
        let y = j as f32;
        let z = 0.0;
        let w = 1.0;
        data.push(PostVertexRaw {
            position: [x, y, z, w],
        })
    }

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("post_vertex_buffer"),
        contents: bytemuck::cast_slice(data.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let count = corners.len() as u32;

    (buffer, count)
}

fn create_framebuffer(
    label: &str,
    device: &wgpu::Device,
    width: u32,
    height: u32,
    color_format: wgpu::TextureFormat,
) -> wgpu::TextureView {
    let texture_label = String::from(label) + "_texture";
    let view_label = String::from(label) + "_view";
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&texture_label),
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: color_format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
    });

    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&view_label),
        ..Default::default()
    })
}

fn create_sampler(label: &str, device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

fn create_pipeline(
    label: &str,
    device: &wgpu::Device,
    binding_layouts: &[&wgpu::BindGroupLayout],
    shader_module: &wgpu::ShaderModule,
    fragment_entry: &str,
    color_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let pipeline_layout_label = String::from(label) + "_pipeline_layout";
    let pipeline_label = String::from(label) + "_pipeline";
    let layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&pipeline_layout_label),
            bind_group_layouts: binding_layouts,
            push_constant_ranges: &[],
        });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&pipeline_label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[PostVertexRaw::desc()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: fragment_entry,
            targets: &[wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        multiview: None,
    })
}

impl Post {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
        static_binding_layout: &wgpu::BindGroupLayout,
        frame_binding_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // Uniform Buffer
        let uniform_buffer = create_uniform_buffer(device);
        // Vertex Buffer
        let (vertex_buffer, vertex_count) = create_vertex_buffer(device);

        // Framebuffers
        let ldr_color = create_framebuffer(
            "ldr_color",
            device,
            width,
            height,
            crate::LDR_COLOR_PIXEL_FORMAT,
        );
        let bright_color = create_framebuffer(
            "bright_color",
            device,
            width,
            height,
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        let ping = create_framebuffer(
            "ping",
            device,
            width,
            height,
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        let pong = create_framebuffer(
            "pong",
            device,
            width,
            height,
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );

        // Framebuffer samplers
        let ldr_color_sampler = create_sampler("ldr_color_sampler", device);
        let bright_color_sampler =
            create_sampler("bright_color_sampler", device);
        let ping_sampler = create_sampler("ping_sampler", device);
        let pong_sampler = create_sampler("pong_sampler", device);

        // Bind Group Layouts
        let blur_pass_bindings = binding::BlurPassBindings::new(device);
        let composite_pass_bindings =
            binding::CompositePassBindings::new(device);

        // Bind Groups
        let hblur0_bind_group = blur_pass_bindings.create_bind_group(
            device,
            uniform_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&bright_color),
            wgpu::BindingResource::Sampler(&bright_color_sampler),
        );
        let hblur1_bind_group = blur_pass_bindings.create_bind_group(
            device,
            uniform_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&ping),
            wgpu::BindingResource::Sampler(&ping_sampler),
        );
        let vblur_bind_group = blur_pass_bindings.create_bind_group(
            device,
            uniform_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&pong),
            wgpu::BindingResource::Sampler(&pong_sampler),
        );
        let composite_bind_group = composite_pass_bindings.create_bind_group(
            device,
            uniform_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&ldr_color),
            wgpu::BindingResource::Sampler(&ldr_color_sampler),
            wgpu::BindingResource::TextureView(&ping),
            wgpu::BindingResource::Sampler(&ping_sampler),
        );

        // Shader
        let shader = wgpu::include_wgsl!("post_shaders.wgsl");
        let shader_module = device.create_shader_module(&shader);

        // Pipelines

        let hblur_pipeline = create_pipeline(
            "horizontal_blur",
            device,
            &[
                static_binding_layout,
                frame_binding_layout,
                &blur_pass_bindings.layout,
            ],
            &shader_module,
            "fs_horizontal_blur_main",
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        let vblur_pipeline = create_pipeline(
            "vertical_blur",
            device,
            &[
                static_binding_layout,
                frame_binding_layout,
                &blur_pass_bindings.layout,
            ],
            &shader_module,
            "fs_vertical_blur_main",
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        let composite_pipeline = create_pipeline(
            "composite",
            device,
            &[
                static_binding_layout,
                frame_binding_layout,
                &composite_pass_bindings.layout,
            ],
            &shader_module,
            "fs_composite_main",
            color_format,
        );

        let hblur0_pass = PostPass {
            render_pass_label: String::from("horizontal_blur_0_render_pass"),
            bind_group_index: binding::BlurPassBindings::GROUP_INDEX,
            bind_group: hblur0_bind_group,
        };

        let hblur1_pass = PostPass {
            render_pass_label: String::from("horizontal_blur_1_render_pass"),
            bind_group_index: binding::BlurPassBindings::GROUP_INDEX,
            bind_group: hblur1_bind_group,
        };

        let vblur_pass = PostPass {
            render_pass_label: String::from("vertical_blur_render_pass"),
            bind_group_index: binding::BlurPassBindings::GROUP_INDEX,
            bind_group: vblur_bind_group,
        };

        let composite_pass = PostPass {
            render_pass_label: String::from("composite_render_pass"),
            bind_group_index: binding::CompositePassBindings::GROUP_INDEX,
            bind_group: composite_bind_group,
        };

        Self {
            vertex_buffer,
            vertex_count,
            uniform_buffer,
            ldr_color,
            bright_color,
            ping,
            pong,
            blur_pass_bindings,
            composite_pass_bindings,
            hblur_pipeline,
            vblur_pipeline,
            composite_pipeline,
            hblur0_pass,
            hblur1_pass,
            vblur_pass,
            composite_pass,
        }
    }

    pub fn input_framebuffer(&self) -> &wgpu::TextureView {
        &self.ldr_color
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.ldr_color = create_framebuffer(
            "ldr_color",
            device,
            width,
            height,
            crate::LDR_COLOR_PIXEL_FORMAT,
        );
        self.bright_color = create_framebuffer(
            "bright_color",
            device,
            width,
            height,
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        self.ping = create_framebuffer(
            "ping",
            device,
            width,
            height,
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        self.pong = create_framebuffer(
            "pong",
            device,
            width,
            height,
            crate::BRIGHT_COLOR_PIXEL_FORMAT,
        );
        let ldr_color_sampler = create_sampler("ldr_color_sampler", device);
        let bright_color_sampler =
            create_sampler("bright_color_sampler", device);
        let ping_sampler = create_sampler("ping_sampler", device);
        let pong_sampler = create_sampler("pong_sampler", device);
        self.hblur0_pass.bind_group =
            self.blur_pass_bindings.create_bind_group(
                device,
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.bright_color),
                wgpu::BindingResource::Sampler(&bright_color_sampler),
            );
        self.hblur1_pass.bind_group =
            self.blur_pass_bindings.create_bind_group(
                device,
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.ping),
                wgpu::BindingResource::Sampler(&ping_sampler),
            );
        self.vblur_pass.bind_group = self.blur_pass_bindings.create_bind_group(
            device,
            self.uniform_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&self.pong),
            wgpu::BindingResource::Sampler(&pong_sampler),
        );
        self.composite_pass.bind_group =
            self.composite_pass_bindings.create_bind_group(
                device,
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.ldr_color),
                wgpu::BindingResource::Sampler(&ldr_color_sampler),
                wgpu::BindingResource::TextureView(&self.ping),
                wgpu::BindingResource::Sampler(&ping_sampler),
            );
    }

    pub fn render(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        image_out: &wgpu::TextureView,
        other_bind_groups: &[&wgpu::BindGroup],
    ) {
        // First blur pass, we read from ldr_color.
        // Subsequent blur passes, we read from pong.
        let mut hblur_pass = &self.hblur0_pass;
        for _ in 0..BLUR_STEPS {
            self.render_post_pass(
                encoder,
                &self.hblur_pipeline,
                hblur_pass,
                &self.pong,
                other_bind_groups,
            );
            self.render_post_pass(
                encoder,
                &self.vblur_pipeline,
                &self.vblur_pass,
                &self.ping,
                other_bind_groups,
            );
            hblur_pass = &self.hblur1_pass;
        }
        self.render_post_pass(
            encoder,
            &self.composite_pipeline,
            &self.composite_pass,
            image_out,
            other_bind_groups,
        );

        // for some number of blur steps:
        //     create render pass
        //     attach pipeline
        //     set bind group
        //     draw a big rectangle (or hexagon)
        //     switch buffers
        // do final composite, tone mapping, and gamma pass
        //     (create render pass, attach pipeline, set bind group)
        //     draw whole screen

        // There are three framebuffers, ldr_color, ping and pong.
        // the forward pass renders into ldr_color.
        // the first horizontal blur copies ldr_color to ping.
        // each vertical blur copies pong to ping.
        // each subsequent horizontal blur copies ping to pong.
        // the final composite copies ping to the output view.
    }

    fn render_post_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        pass: &PostPass,
        image_out: &wgpu::TextureView,
        other_bind_groups: &[&wgpu::BindGroup],
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&pass.render_pass_label),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: image_out,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        render_pass.set_pipeline(pipeline);
        for (i, bg) in other_bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bg, &[]);
        }
        render_pass.set_bind_group(
            pass.bind_group_index,
            &pass.bind_group,
            &[],
        );
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
