use anyhow::{anyhow, Result};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

const WINDOW_W: u32 = 640;
const WINDOW_H: u32 = 480; // 360;

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn render(&mut self) {
        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        
        // Renders a GREEN screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    windows: Vec<State>, // space for future use, by example surface texture
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        let window_attr = Window::default_attributes()
            .with_title("Window")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(WINDOW_W, WINDOW_H));
        let window = Arc::new(event_loop.create_window(window_attr).unwrap());
        let state = pollster::block_on(State::new(window));
        self.windows.push(state);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        println!("suspended");
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("exiting");
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // todo ???
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\tCloseRequested");
                self.windows.retain(|state| state.get_window().id() != id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => match event.physical_key {
                PhysicalKey::Code(code) => {
                    println!(
                        "\tKeyboardInput {:?} - {:?} - {}",
                        code, event.state, is_synthetic
                    );
                    if !is_synthetic {
                        // windows[idx].on_key_input(code, event.state == ElementState::Pressed, &q);
                    }
                    match code {
                        KeyCode::Escape => {
                            self.windows.retain(|state| state.get_window().id() != id);
                            if self.windows.is_empty() {
                                event_loop.exit();
                            }
                        }
                        KeyCode::KeyF => {
                            println!("KeyF");
                            // toggle fullscreen
                            if let Some(state) = self
                                .windows
                                .iter_mut()
                                .find(|state| state.get_window().id() == id)
                            {
                                state.get_window().set_fullscreen(if state.get_window().fullscreen().is_some() {
                                    None
                                } else {
                                    Some(Fullscreen::Borderless(None))
                                });
                            }
                        }
                        _ => {
                            // println!("Other");
                        }
                    }
                }
                _ => {}
            },
            // WindowEvent::ActivationTokenDone { serial, token } => todo!(),
            WindowEvent::Resized(new_size) => {
                println!("\tResized {:?}", new_size);
                if let Some(state) = self.windows.iter_mut().find(|state| state.get_window().id() == id) {
                    state.resize(new_size);
                }
            }
            // WindowEvent::Moved(_) => todo!(),
            WindowEvent::Destroyed => {
                println!("\tDestroyed");
            }
            // WindowEvent::DroppedFile(_) => todo!(),
            // WindowEvent::HoveredFile(_) => todo!(),
            // WindowEvent::HoveredFileCancelled => todo!(),
            WindowEvent::Focused(focused) => {
                println!("\tFocused {}", focused);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                println!("\tModifiersChanged {:?}", modifiers);
            }
            // WindowEvent::Ime(_) => todo!(),
            // WindowEvent::CursorMoved { device_id, position } => todo!(),
            // WindowEvent::CursorEntered { device_id } => todo!(),
            // WindowEvent::CursorLeft { device_id } => todo!(),
            WindowEvent::MouseWheel { delta, .. } => {
                println!("\tMouseWheel {:?}", delta);
            }
            WindowEvent::MouseInput { button, .. } => {
                println!("\tMouseInput {:?}", button);
            }
            // WindowEvent::TouchpadMagnify { device_id, delta, phase } => todo!(),
            // WindowEvent::SmartMagnify { device_id } => todo!(),
            // WindowEvent::TouchpadRotate { device_id, delta, phase } => todo!(),
            // WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
            // WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
            // WindowEvent::Touch(_) => todo!(),
            // WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => todo!(),
            // WindowEvent::ThemeChanged(_) => todo!(),
            WindowEvent::Occluded(occluded) => {
                // not raised on Windows 11
                println!("\tOccluded {}", occluded);
            }
            WindowEvent::RedrawRequested => {
                println!("\tRedrawRequested");
                if let Some(state) = self.windows.iter_mut().find(|state| state.get_window().id() == id) {
                    state.render();
                    state.get_window().request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).map_err(|err| anyhow!(err))
}
