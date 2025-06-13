use anyhow::{anyhow, Result};
use std::{num::NonZeroU32, time::Instant};
use std::rc::Rc;
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

#[derive(Default)]
struct App {
    windows: Vec<(
        Rc<Window>,
        softbuffer::Context<Rc<Window>>,
        softbuffer::Surface<Rc<Window>, Rc<Window>>,
    )>,
    x: i32,
    inc_x: i32,
    prev: Option<Instant>,
}

impl App {
    fn redraw(&mut self, id: WindowId) {
        if self.inc_x == 0 {
            self.inc_x = 1;
        }

        if let Some(prev) = self.prev {
            let dt = prev.elapsed();
            println!("dt: {:?} fps: {}", dt, 1.0 / dt.as_secs_f64());
        }
        self.prev = Some(Instant::now());

        self.x += self.inc_x;
        println!("x: {}, inc_x: {}", self.x, self.inc_x);

        if self.x > 400 {
            self.inc_x = -1;
        } else if self.x < 100 {
            self.inc_x = 1;
        }

        let (window, _context, surface) = self
            .windows
            .iter_mut()
            .find(|(window, _, _)| window.id() == id)
            .unwrap();
        let (width, height) = {
            let size = window.inner_size();
            (size.width, size.height)
        };
        surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();

        let scale_x = (width as f32 / WINDOW_W as f32).floor() as u32;
        let scale_y = (height as f32 / WINDOW_H as f32).floor() as u32;
        let scale = std::cmp::min(scale_x, scale_y);

        let ww = WINDOW_W * scale;
        let wh = WINDOW_H * scale;
        let scale = scale as i32;

        let mut buffer = surface.buffer_mut().unwrap();
        for index in 0..(width * height) {
            let p_y = (index / width) as i32;
            let p_x = (index % width) as i32;

            let x = (p_x - ((width - ww) as i32 / 2)) / scale;
            let y = (p_y - ((height - wh) as i32 / 2)) / scale;

            let (red, green, blue) =
                if x < 0 || x >= WINDOW_W as i32 || y < 0 || y >= WINDOW_H as i32 {
                    (0, 0, 0)
                // } else if (x >= 100 && x <= 104) || (y >= 100 && y <= 104) {
                } else if (x == self.x) || (y == self.x) {
                    (255, 255, 255)
                } else {
                    (0, 80, 130)
                };

            buffer[index as usize] = blue | (green << 8) | (red << 16);
        }

        buffer.present().unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        let window_attr = Window::default_attributes()
            .with_title("Window")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(WINDOW_W, WINDOW_H));
        let window = Rc::new(event_loop.create_window(window_attr).unwrap());
        let context = softbuffer::Context::new(Rc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Rc::clone(&window)).unwrap();
        self.windows.push((window, context, surface));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        println!("suspended");
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("exiting");
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window_ids: Vec<_> = self.windows.iter().map(|(window, _, _)| window.id()).collect();
        for id in window_ids {
            self.redraw(id);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\tCloseRequested");
                self.windows.retain(|(window, _, _)| window.id() != id);
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
                            self.windows.retain(|(window, _, _)| window.id() != id);
                            if self.windows.is_empty() {
                                event_loop.exit();
                            }
                        }
                        KeyCode::KeyF => {
                            println!("KeyF");
                            // toggle fullscreen
                            if let Some((window, _, _)) = self
                                .windows
                                .iter_mut()
                                .find(|(window, _, _)| window.id() == id)
                            {
                                window.set_fullscreen(if window.fullscreen().is_some() {
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
                self.redraw(id);
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
