// Binding
//
// enums(?) defining the bind groups for each pass
// BIND_GROUP_STATIC
// BIND_GROUP_FRAME
// BIND_GROUP_EPHEMERAL

#[allow(dead_code)]
pub struct Bg(pub u32);

impl Bg {
    pub const STATIC: Bg = Bg { 0: 0 };
    pub const FRAME: Bg = Bg { 0: 1 };
}

// struct Binding(u32);
// impl Binding {

//     // Static bindings
//     const FACE_DECAL: Binding = Binding { 0: 0 };
//     const CAMERA_UNIFORM: Binding = Binding { 0: 1 };
//     const LIGHTS_UNIFORM: Binding = Binding { 0: 2 };

//     // Frame bindings
//     const BLINKY_TEXTURE: Binding = Binding { 0: 0 };
//     const CUBE_UNIFORM: Binding = Binding { 0: 1 };
// }

pub struct StaticBg {
    pub layout: wgpu::BindGroupLayout,
}

impl StaticBg {
    const FACE_DECAL: u32 = 0;
    const CAMERA_UNIFORM: u32 = 1;
    // const LIGHTS_UNIFORM: u32 = 2;

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
                ],
            }
        );
        Self {
            layout,
        }
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        face_decal: wgpu::BindingResource,
        camera_uniform: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
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
                ],
            }
        )
    }
}

pub struct FrameBg {
    pub layout: wgpu::BindGroupLayout,
}

impl FrameBg {
    const BLINKY_TEXTURE: u32 = 0;
    const CUBE_UNIFORM: u32 = 1;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                        visibility: wgpu::ShaderStages::VERTEX |
                            wgpu::ShaderStages::FRAGMENT,
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
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
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
            }
        )
    }
}

////////////////////////////////////////////////////////////////////////

// trait Binder {
//     fn get_resource(&self, name: &str);
// }

// struct Variable {
//     name: String,
//     layout: wgpu::BindGroupLayoutEntry,
//     // binder: fn () -> wgpu::BindingResource<'a>,
// }

// impl Variable {
//     pub fn new(name: &str, layout: wgpu::BindGroupLayoutEntry) -> Self {
//         Self {
//             name: String::from(name),
//             layout,
//         }
//     }
// }

// type Directory = std::collections::HashMap<String, Variable>;

// struct BindGroup {
//     group_name: String,
//     var_names: Vec<String>,
//     layout_label: String,
//     group_label: String,
// }

// impl BindGroup {
//     fn new(group_name: &str, var_names: &[&str]) -> Self {
//         Self {
//             group_name: String::from(group_name),
//             var_names: var_names.iter().map(|s| String::from(*s)).collect(),
//             layout_label: format!("{}_bind_group_layout", group_name),
//             group_label: format!("{}_bind_group", group_name),
//         }
//     }
//     fn create_layout(
//         self,
//         device: &wgpu::Device,
//         directory: &Directory,
//     ) -> wgpu::BindGroupLayout {
//         let label = Some(self.layout_label.as_str());
//         let entry_vec = self
//             .var_names
//             .iter()
//             .enumerate()
//             .map(|(i, name)| {
//                 let var = directory.get(name).unwrap();
//                 let mut entry = var.layout.clone();
//                 entry.binding = i as u32;
//                 entry
//             })
//             .collect::<Vec<_>>();
//         let entries = entry_vec.as_slice();

//         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label,
//             entries,
//         })
//     }

//     fn create_bind_group<'a>(
//         self,
//         device: &wgpu::Device,
//         variables: &Directory,
//         resources: &[wgpu::BindingResource<'a>],
//     ) -> wgpu::BindGroup {
//         let label = Some(self.group_label.as_str());
//         assert!(self.var_names.len() == resources.len());
//         let layout = self.create_layout(device, variables);
//         let entry_vec = resources
//             .iter()
//             .enumerate()
//             .map(|(i, resource)| {
//                 wgpu::BindGroupEntry {
//                     binding: i as u32,
//                     resource: *resource,
//                 }
//             })
//             .collect::<Vec<_>>();
//         let entries = entry_vec.as_slice();
//         device.create_bind_group(
//             &wgpu::BindGroupDescriptor {
//                 label,
//                 layout: &layout,
//                 entries,
//             }
//         )
//     }
// }

////////////////////////////////////////////////////////////////////////


// struct StaticBg ();

// impl StaticBg {
//     const FACE_DECAL: usize = 0;
//     const CAMERA_UNIFORM: usize = 1;
//     const LIGHTS_UNIFORM: usize = 2;
// }

// pub struct FrameBg {
//     pub layout: wgpu::BindGroupLayout,
// }

// impl FrameBg {
//     const BLINKY_TEXTURE: u32 = 0;
//     const CUBE_UNIFORM: u32 = 1;

//     fn new(device: &wgpu::Device) -> Self {
//         let layout = device.create_bind_group_layout(
//             &wgpu::BindGroupLayoutDescriptor {
//                 label: Some("frame_bind_group_layout"),
//                 entries: &[
//                     wgpu::BindGroupLayoutEntry {
//                         binding: Self::BLINKY_TEXTURE,
//                         visibility: wgpu::ShaderStages::FRAGMENT,
//                         ty: wgpu::BindingType::Texture {
//                             multisampled: false,
//                             view_dimension: wgpu::TextureViewDimension::D2,
//                             sample_type: wgpu::TextureSampleType::Uint,
//                         },
//                         count: None,
//                     },
//                     wgpu::BindGroupLayoutEntry {
//                         binding: Self::CUBE_UNIFORM,
//                         visibility: (
//                             wgpu::ShaderStages::VERTEX |
//                             wgpu::ShaderStages::FRAGMENT
//                         ),
//                         ty: wgpu::BindingType::Buffer {
//                             ty: wgpu::BufferBindingType::Uniform,
//                             has_dynamic_offset: false,
//                             min_binding_size: None,
//                         },
//                         count: None,
//                     },
//                 ],
//             }
//         );
//         Self {
//             layout,
//         }
//     }

//     fn create_bind_group(
//         &self,
//         device: &wgpu::Device,
//         // blinky: &crate::texture::Texture, 
//         blinky: wgpu::BindingResource,
//         cube_uniform: &wgpu::Buffer
//     ) -> wgpu::BindGroup {
//         device.create_bind_group(
//             &wgpu::BindGroupDescriptor {
//                 label: Some("static_bind_group"),
//                 layout: &self.layout,
//                 entries: &[
//                     wgpu::BindGroupEntry {
//                         binding: Self::BLINKY_TEXTURE,
//                         resource: blinky,
//                         // resource: wgpu::BindingResource::TextureView(
//                         //     &blinky.view,
//                         // ),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: Self::CUBE_UNIFORM,
//                         resource: cube_uniform.as_entire_binding(),
//                     },
//                 ],
//             }
//         )
//     }
// }
