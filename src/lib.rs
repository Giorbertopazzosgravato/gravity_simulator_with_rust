use cgmath::prelude::*;
use std::{iter};
use std::default::Default;
use cgmath::Vector3;
use image::GenericImageView;
use rand::Rng;
use wgpu::{Buffer, BufferUsages, Color, DeviceDescriptor, Features, RenderPipeline, Surface, SurfaceError};
use wgpu::IndexFormat::Uint16;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use crate::black_hole::BlackHole;
use crate::instance::{Instance, InstanceRaw};
use crate::vertex::Vertex;

mod vertex;
mod create_perfect_shape;
mod instance;
mod black_hole;
mod utils;
struct State {
    surface: Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    instances_render_pipeline: RenderPipeline,
    black_hole_render_pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    instances: Vec<Instance>,
    instance_buffer: Buffer,
    mouse_x: f32,
    mouse_y: f32,
    is_lmb_pressed: bool,

    black_hole: BlackHole,
}

impl State {
    async unsafe fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor{
                features: Features::POLYGON_MODE_LINE | Features::POLYGON_MODE_POINT,
                limits: wgpu::Limits::default(),
                label: Some("aha aha aha"),
            },
            None
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);


        let instances_render_pipeline: RenderPipeline = utils::Utils::create_pipeline(
            "./shaders/instance_shader.wgsl",
            &device,
            &config,
            &[
                Vertex::desc(),
                InstanceRaw::desc()
            ]);
        let black_hole_render_pipeline: RenderPipeline = utils::Utils::create_pipeline(
            "./shaders/black_hole_shader.wgsl",
            &device,
            &config,
            &[Vertex::desc()]
        );

        let (vertices, indices) = create_perfect_shape::CreatePerfectShape::create_shape_optimized(8, [1.0, 0.0, 0.0]);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        let instances: Vec<Instance> = Vec::new();
        let raw_instances = instances.iter().map(|instance| instance.to_raw()).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(&raw_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        let black_hole = BlackHole::new(&device);
        Self {
            surface,
            device,
            queue,
            size,
            config,
            instances_render_pipeline,
            black_hole_render_pipeline,
            window,

            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,

            instances,
            instance_buffer,
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_lmb_pressed: false,
            black_hole,
        }
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { device_id, state, button, .. } =>{
                match button {
                    MouseButton::Left => {
                        if state == &ElementState::Pressed {
                            self.is_lmb_pressed = true;
                        }
                        if state == &ElementState::Released {
                            self.is_lmb_pressed = false;
                        }
                    }
                    _ => {

                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id, position, ..
            } => {
                let window_size = self.window().inner_size();
                let (x, y) = (position.x / (window_size.width as f64 / 2.0) - 1.0, 1.0 - position.y / (window_size.height as f64 / 2.0));
                self.mouse_x = x as f32;
                self.mouse_y = y as f32;
            }
            _ => {
            }
        }
        return self.is_lmb_pressed;
    }
    fn update(&mut self) {
        Self::process_user_input(self);

        for instance in &mut self.instances {
            instance.update_forces(instance.calculate_gravitational_pull());
            //println!("forces: {:?}", instance.forces)
        }
        let raw_instances = self.instances.iter().map(|inst| inst.to_raw()).collect::<Vec<_>>();
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&raw_instances));
    }
    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = &output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.instances_render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), Uint16);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as u32);

            let (vertex_buffer, index_buffer, num_indices) = self.black_hole.get_buffers();
            render_pass.set_pipeline(&self.black_hole_render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);

        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
    fn process_user_input(&mut self){
        if self.is_lmb_pressed {
            self.instances.push(Instance{
                position: Vector3 {
                    x: self.mouse_x,
                    y: self.mouse_y,
                    z: 0.0,
                },
                color: Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 0.5,
                },
                forces: Vector3 {
                    x: rand::thread_rng().gen_range(-0.05..=0.05),
                    y: rand::thread_rng().gen_range(-0.05..=0.05),
                    // x: 0.0,
                    // y: 0.0,
                    z: 0.0,
                },
            });
            let instance_data = self.instances.iter().map(|instance| instance.to_raw()).collect::<Vec<_>>();
            self.instance_buffer = self.device.create_buffer_init(&BufferInitDescriptor{
                label: Some("instance buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(600, 600))
        .build(&event_loop).unwrap();
    // State::new uses async code, so we're going to wait for it to finish
    let mut state = unsafe { State::new(window) }.await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}