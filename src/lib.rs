mod controller;
mod geometry;
mod init;
mod light;
mod model;
mod pipelines;
mod resources;
mod texture;
mod transforms;
mod view;

use init::init;
use pipelines::{create_light_render_pipeline, create_model_render_pipeline};
use std::mem;
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

    light: light::Light,
    light_render_pipeline: wgpu::RenderPipeline,

    model: model::Model,
    model_render_pipeline: wgpu::RenderPipeline,

    depth_texture: texture::Texture,

    instances: transforms::Transforms,
}

impl State {
    async fn new(window: &Window) -> State {
        let (surface, device, queue, config) = init(window).await;

        let view = view::View::build()
            .camera((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0))
            .projection(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0)
            .controller(controller::Controller::new(4.0, 0.4))
            .finalize(&device);

        let light = light::Light::build()
            .position([4.0, 4.0, 2.0])
            .color([1.0, 1.0, 1.0])
            .controller(60.0)
            .finalize(&device);

        let light_render_pipeline = create_light_render_pipeline(&device, &config, &view, &light);

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture",
        );

        let model = resources::load_model("cube.obj", &device, &queue)
            .await
            .unwrap();

        let model_render_pipeline =
            create_model_render_pipeline(&device, &config, &model, &view, &light);

        let instances = transforms::Transforms::build()
            .transform_field(3, 3)
            .finalize(&device);

        Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            view,
            instances,
            model_render_pipeline,
            model,
            light,
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
        // also update the buffer and adds it to the queue
        self.view.update(dt, &self.queue);
        self.light.update(dt, &self.queue);
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

        render_pass.set_pipeline(&self.model_render_pipeline);

        render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));

        render_pass.set_bind_group(1, &self.view.bind_group, &[]);
        render_pass.set_bind_group(2, &self.light.bind_group, &[]);

        for mesh in &self.model.meshes {
            let material = &self.model.materials[mesh.material];

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &material.bind_group, &[]);

            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.instances.number);
        }

        render_pass.set_pipeline(&self.light_render_pipeline);
        for mesh in &self.model.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.view.bind_group, &[]);
            render_pass.set_bind_group(1, &self.light.bind_group, &[]);
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
                        }
                    }
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
