mod controller;
mod geometry;
mod init;
mod instances;
mod light;
mod model;
mod resources;
mod texture;
mod view;

use init::init;
use light::UpdateBuffer;
use std::mem;
use wgpu::util::DeviceExt;
use winit::keyboard::{Key, NamedKey, PhysicalKey};
use winit::{event::*, event_loop::EventLoop, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    view: view::View,

    light_controller: light::LightController,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,

    model: model::Model,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(window: &Window) -> State {
        let (surface, device, queue, config) = init(window).await;

        let view = view::View::build()
            .camera((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0))
            .projection(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0)
            .controller(controller::Controller::new(4.0, 0.4))
            .finalize(&device);

        /* Light */

        let light = light::Light::new([4.0, 4.0, 2.0], [1.0, 1.0, 1.0]);

        let light_buffer = light::LightBuffer::new(&device, &light);

        let light_controller = light::LightController::new(60.0, light);

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.buffer.as_entire_binding(),
            }],
            label: None,
        });

        let light_render_pipeline = {
            let bind_groups_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Light Pipeline Layout"),
                    bind_group_layouts: &[&view.bind_group_layout, &light_bind_group_layout],
                    push_constant_ranges: &[],
                });

            let vertex_layout = wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<model::ModelVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                        shader_location: 3,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                        shader_location: 4,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                ],
            };

            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };

            let shader = device.create_shader_module(shader);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&bind_groups_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_layout],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        };

        /* Geometry */

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    // diffuse map
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture",
        );

        let model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();

        let transforms = instances::sample_transform_field(3, 3);

        let instance_data = transforms
            .iter()
            .map(instances::Transform::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_instances = transforms.len() as u32;

        let render_pipeline = {
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout, // group(0)
                        &view.bind_group_layout,    // group(1)
                        &light_bind_group_layout,   // group(2)
                    ],
                    // render_pass.set_bind_group(0, &material.bind_group, &[]);
                    // render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    // render_pass.set_bind_group(2, &self.light_bind_group, &[]);
                    push_constant_ranges: &[],
                });

            let vertex_layout = wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<model::ModelVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0, // position
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        shader_location: 1, // tex_coords
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                        shader_location: 2, // normal
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                        shader_location: 3, // tangent
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                        shader_location: 4, // bitangent
                        format: wgpu::VertexFormat::Float32x3,
                    },
                ],
            };

            let instances_layout = wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<instances::RawInstance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 5, // 1st row of  model transformation matrix
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        shader_location: 6, // 2nd row of model transformation matrix
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                        shader_location: 7, // 3rd row of model transformation matrix
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                        shader_location: 8, // 4th row of model transformation matrix
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                        shader_location: 9, // 1st row of normal transformation matrix
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                        shader_location: 10, // 2nd row of normal transformation matrix
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                        shader_location: 11, // 3rd row of normal transformation matrix
                        format: wgpu::VertexFormat::Float32x3,
                    },
                ],
            };

            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };

            let shader = device.create_shader_module(shader);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_layout, instances_layout],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        };

        Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            view,
            instance_buffer,
            num_instances,
            render_pipeline,
            model,
            light_controller,
            light_buffer: light_buffer.buffer,
            light_bind_group,
            light_render_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                new_size.width,
                new_size.height,
                "depth_texture",
            );

            self.surface.configure(&self.device, &self.config);

            self.view.projection.resize(new_size.width, new_size.height);
        }
    }

    pub fn reset(&mut self) {
        self.resize(winit::dpi::PhysicalSize::<u32> {
            width: self.config.width,
            height: self.config.height,
        })
    }

    fn update(&mut self, dt: instant::Duration) {
        // updates camera position
        self.view.update(dt, &self.queue);

        // updates light position
        self.light_controller.update(dt);

        self.light_controller
            .light
            .update(&self.queue, &self.light_buffer);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_bind_group(1, &self.view.bind_group, &[]);
        render_pass.set_bind_group(2, &self.light_bind_group, &[]);

        for mesh in &self.model.meshes {
            let material = &self.model.materials[mesh.material];

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &material.bind_group, &[]);

            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.num_instances);
        }

        render_pass.set_pipeline(&self.light_render_pipeline);
        for mesh in &self.model.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.view.bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }

        drop(render_pass);

        let command_buffer = encoder.finish();

        self.queue.submit([command_buffer]);

        output.present();

        Ok(())
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut state = State::new(&window).await;

    let mut last_render_time = instant::Instant::now();

    // window.set_cursor_visible(false);
    // window
    //     .set_cursor_grab(winit::window::CursorGrabMode::Locked)
    //     .unwrap();

    event_loop
        .run(move |event, elwt| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => state.view.controller.process_mouse(delta.0, delta.1),
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => elwt.exit(),
                WindowEvent::Resized(physical_size) => {
                    cfg_if::cfg_if! {
                    if #[cfg(not(target_arch = "wasm32"))]{
                        state.resize(*physical_size)
                    }}
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state: keyboard_state,
                            ..
                        },
                    ..
                } => state
                    .view
                    .controller
                    .process_keyboard(*key, *keyboard_state),
                WindowEvent::MouseWheel { delta, .. } => {
                    state.view.controller.process_scroll(delta)
                }

                WindowEvent::RedrawRequested => {
                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    state.update(dt);
                    match state.render() {
                        Ok(_) => window.request_redraw(),
                        Err(wgpu::SurfaceError::Lost) => state.reset(),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        Err(wgpu::SurfaceError::Outdated) => log::warn!("Surface outdated"),
                    }
                }
                _ => (),
            },
            _ => (),
        })
        .unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")]{
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = winit::window::Window::new(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS`dd
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(128, 256));

        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas().unwrap();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.append_child(&canvas).unwrap();
    }
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")]{
            wasm_bindgen_futures::spawn_local(run(event_loop, window));
        } else {
            pollster::block_on(run(event_loop, window));
        }
    }
}
