use crate::binding;
use crate::bounds;
use wgpu::util::DeviceExt;

const BLUR_STEPS: usize = 3;
const SCALING_STEPS: usize = 3;
const PASS_COUNT: usize = 2 * BLUR_STEPS + 1;
const BLUR_RADIUS: u32 = (4 * BLUR_STEPS << SCALING_STEPS) as _;

const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniformRaw {
    image_size: [f32; 2],
    output_size: [f32; 2],
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

#[derive(Copy, Clone, Debug)]
pub struct Configuration {
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl Configuration {
    fn with_format(&self, format: wgpu::TextureFormat) -> Self {
        Self {
            width: self.width,
            height: self.height,
            format,
        }
    }
}

pub struct Post {
    config: Configuration,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_aligned_size: usize,
    ldr_color: wgpu::TextureView,
    ping: wgpu::TextureView,
    pong: wgpu::TextureView,
    blur_pass_bindings: binding::BlurPassBindings,
    composite_pass_bindings: binding::CompositePassBindings,
    hblur_pipeline: wgpu::RenderPipeline,
    vblur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    hblur_pass: PostPass,
    vblur_pass: PostPass,
    composite_pass: PostPass,
}

fn round_up(n: usize, align: u32) -> usize {
    let align = align as usize;
    (n + align - 1) / align * align
}

fn create_uniform_buffer(device: &wgpu::Device) -> (wgpu::Buffer, usize) {
    let raw_size = std::mem::size_of::<PostUniformRaw>();
    let min_align = device.limits().min_uniform_buffer_offset_alignment;
    let aligned_size = round_up(raw_size, min_align);
    let buffer_size = PASS_COUNT * aligned_size;
    let mut data = vec![0u8; buffer_size];

    let mut insert = |pos: usize, post: PostUniformRaw| {
        let offset = pos * aligned_size;
        let end = offset + raw_size;
        *bytemuck::from_bytes_mut::<PostUniformRaw>(&mut data[offset..end]) =
            post;
    };

    let mut image_size = [1.0, 1.0];
    let mut output_size = [1.0, 1.0];
    for i in 0..BLUR_STEPS as usize {
        if i < SCALING_STEPS {
            output_size[0] *= 0.5;
        }
        insert(
            2 * i,
            PostUniformRaw {
                image_size,
                output_size,
            },
        );
        if i < SCALING_STEPS {
            image_size[0] *= 0.5;
            output_size[1] *= 0.5;
        }
        insert(
            2 * i + 1,
            PostUniformRaw {
                image_size,
                output_size,
            },
        );
        if i < SCALING_STEPS {
            image_size[1] *= 0.5;
        }
    }
    insert(
        2 * BLUR_STEPS,
        PostUniformRaw {
            image_size,
            output_size: [1.0, 1.0],
        },
    );
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("post_uniform_buffer"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    (buffer, aligned_size)
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
    config: &Configuration,
) -> wgpu::TextureView {
    let texture_label = String::from(label) + "_texture";
    let view_label = String::from(label) + "_view";
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&texture_label),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
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
        config: &Configuration,
        static_binding_layout: &wgpu::BindGroupLayout,
        frame_binding_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let config = *config;
        // Uniform Buffer
        let (uniform_buffer, uniform_aligned_size) =
            create_uniform_buffer(device);

        // Vertex Buffer
        let (vertex_buffer, vertex_count) = create_vertex_buffer(device);

        // Framebuffers
        let ldr_color = create_framebuffer(
            "ldr_color",
            device,
            &config.with_format(crate::LDR_COLOR_PIXEL_FORMAT),
        );
        let ping = create_framebuffer(
            "ping",
            device,
            &config.with_format(crate::BRIGHT_COLOR_PIXEL_FORMAT),
        );
        let pong = create_framebuffer(
            "pong",
            device,
            &config.with_format(crate::BRIGHT_COLOR_PIXEL_FORMAT),
        );

        // Framebuffer samplers
        let ldr_color_sampler = create_sampler("ldr_color_sampler", device);
        let ping_sampler = create_sampler("ping_sampler", device);
        let pong_sampler = create_sampler("pong_sampler", device);

        // Bind Group Layouts
        let blur_pass_bindings = binding::BlurPassBindings::new(device);
        let composite_pass_bindings =
            binding::CompositePassBindings::new(device);

        // Bind Groups
        let uniform_resource =
            wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &uniform_buffer,
                offset: 0,
                size: wgpu::BufferSize::new(
                    std::mem::size_of::<PostUniformRaw>() as _,
                ),
            });
        let hblur_bind_group = blur_pass_bindings.create_bind_group(
            device,
            uniform_resource.clone(),
            wgpu::BindingResource::TextureView(&ping),
            wgpu::BindingResource::Sampler(&ping_sampler),
        );
        let vblur_bind_group = blur_pass_bindings.create_bind_group(
            device,
            uniform_resource.clone(),
            wgpu::BindingResource::TextureView(&pong),
            wgpu::BindingResource::Sampler(&pong_sampler),
        );
        let composite_bind_group = composite_pass_bindings.create_bind_group(
            device,
            uniform_resource.clone(),
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
            config.format,
        );

        let hblur_pass = PostPass {
            render_pass_label: String::from("horizontal_blur_render_pass"),
            bind_group_index: binding::BlurPassBindings::GROUP_INDEX,
            bind_group: hblur_bind_group,
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
            config,
            vertex_buffer,
            vertex_count,
            uniform_buffer,
            uniform_aligned_size,
            ldr_color,
            ping,
            pong,
            blur_pass_bindings,
            composite_pass_bindings,
            hblur_pipeline,
            vblur_pipeline,
            composite_pipeline,
            hblur_pass,
            vblur_pass,
            composite_pass,
        }
    }

    pub fn input_framebuffer(&self) -> &wgpu::TextureView {
        &self.ldr_color
    }

    pub fn bright_framebuffer(&self) -> &wgpu::TextureView {
        &self.ping
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.ldr_color = create_framebuffer(
            "ldr_color",
            device,
            &self.config.with_format(crate::LDR_COLOR_PIXEL_FORMAT),
        );
        self.ping = create_framebuffer(
            "ping",
            device,
            &self.config.with_format(crate::BRIGHT_COLOR_PIXEL_FORMAT),
        );
        self.pong = create_framebuffer(
            "pong",
            device,
            &self.config.with_format(crate::BRIGHT_COLOR_PIXEL_FORMAT),
        );
        let ldr_color_sampler = create_sampler("ldr_color_sampler", device);
        let ping_sampler = create_sampler("ping_sampler", device);
        let pong_sampler = create_sampler("pong_sampler", device);
        let uniform_resource =
            wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.uniform_buffer,
                offset: 0,
                size: wgpu::BufferSize::new(
                    std::mem::size_of::<PostUniformRaw>() as _,
                ),
            });
        self.hblur_pass.bind_group = self.blur_pass_bindings.create_bind_group(
            device,
            uniform_resource.clone(),
            wgpu::BindingResource::TextureView(&self.ping),
            wgpu::BindingResource::Sampler(&ping_sampler),
        );
        self.vblur_pass.bind_group = self.blur_pass_bindings.create_bind_group(
            device,
            uniform_resource.clone(),
            wgpu::BindingResource::TextureView(&self.pong),
            wgpu::BindingResource::Sampler(&pong_sampler),
        );
        self.composite_pass.bind_group =
            self.composite_pass_bindings.create_bind_group(
                device,
                uniform_resource.clone(),
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
        bloom_bounds: &bounds::Bounds,
    ) {
        for i in 0..BLUR_STEPS {
            self.render_post_pass(
                encoder,
                &self.hblur_pipeline,
                &self.hblur_pass,
                &self.pong,
                2 * i,
                other_bind_groups,
                Some(bloom_bounds),
            );
            self.render_post_pass(
                encoder,
                &self.vblur_pipeline,
                &self.vblur_pass,
                &self.ping,
                2 * i + 1,
                other_bind_groups,
                Some(bloom_bounds),
            );
        }
        self.render_post_pass(
            encoder,
            &self.composite_pipeline,
            &self.composite_pass,
            image_out,
            2 * BLUR_STEPS,
            other_bind_groups,
            None,
        );
    }

    fn render_post_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        pass: &PostPass,
        image_out: &wgpu::TextureView,
        pass_number: usize,
        other_bind_groups: &[&wgpu::BindGroup],
        bloom_bounds: Option<&bounds::Bounds>,
    ) {
        // const NEXT_LAST: usize = PASS_COUNT - 2;
        // let load_op = match pass_number {
        //     0 => wgpu::LoadOp::Clear(wgpu::Color {
        //         r: 0.0,
        //         g: 1.0,
        //         b: 1.0,
        //         a: 1.0,
        //     }),
        //     1 => wgpu::LoadOp::Clear(wgpu::Color {
        //         r: 1.0,
        //         g: 0.0,
        //         b: 1.0,
        //         a: 1.0,
        //     }),
        //     NEXT_LAST => wgpu::LoadOp::Clear(BLACK),
        //     _ => wgpu::LoadOp::Load,
        // };
        let load_op = wgpu::LoadOp::Clear(BLACK);
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&pass.render_pass_label),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: image_out,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        let mut owf = 1.0 / (1 << (pass_number + 2) / 2) as f32;
        let mut ohf = 1.0 / (1 << (pass_number + 1) / 2) as f32;
        if pass_number == 2 * BLUR_STEPS as usize {
            owf = 1.0;
            ohf = 1.0;
        }
        // println!("pass {} {}x{}", pass_number, w, h);
        // println!(
        //     "x {}, y {}, w {}, h {}",
        //     0,
        //     ((1.0 - h) * 1080.0) as u32,
        //     (w * 1920.0) as u32,
        //     (h * 1080.0) as u32,
        // );
        // XXX guessing
        if let Some(bounds) = bloom_bounds {
            let cw = self.config.width;
            let ch = self.config.height;
            let cwf = cw as f32;
            let chf = ch as f32;
            // let l = (0.1 * owf * cwf) as u32;
            // let r = (0.65 * owf * cwf) as u32;
            // let t = ((1.0 - 0.95 * ohf) * chf as f32) as u32;
            // let b = ((1.0 - 0.02 * ohf) * chf as f32) as u32;
            // println!("old b: {}", b);
            // bounds ops: union, intersection, grow, transform
            // bloom_bounds * viewport_bounds
            let l = 0.max(
                ((bounds.xmin + 1.0) * 0.5 * owf * cwf) as i32
                    - (BLUR_RADIUS as f32 * owf) as i32,
            ) as u32;
            let r = cw.min(
                ((bounds.xmax + 1.0) * 0.5 * owf * cwf) as u32
                    + (BLUR_RADIUS as f32 * owf) as u32,
            );
            let t = 0.max(
                ((1.0 - ((1.0 + bounds.ymax) * 0.5) * ohf) * chf) as i32
                    - (BLUR_RADIUS as f32 * ohf) as i32,
            ) as u32;
            let b = ch.min(
                ((1.0 - ((1.0 + bounds.ymin) * 0.5) * ohf) * chf) as u32
                    + (BLUR_RADIUS as f32 * ohf) as u32,
            );

            // println!("{:?}", bloom_bounds);
            // println!("pass {}: l {}, r {}", pass_number, l, r);
            // println!("        owf {} cw {} cwf {}", owf, cw, cwf);
            // println!("pass {}: t {}, b {}", pass_number, t, b);
            // println!("        ohf {} ch {} chf {}", ohf, ch, chf);
            // println!("        b - t = {}", b - t);
            // println!("BLUR_RADIUS = {}", BLUR_RADIUS);
            render_pass.set_scissor_rect(l, t, r - l, b - t);
        }
        render_pass.set_pipeline(pipeline);
        for (i, bg) in other_bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bg, &[]);
        }
        render_pass.set_bind_group(
            pass.bind_group_index,
            &pass.bind_group,
            &[
                (pass_number * self.uniform_aligned_size)
                    as wgpu::DynamicOffset,
            ],
        );
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
