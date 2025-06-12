use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId, Fullscreen},
    dpi::PhysicalSize,
};
use std::num::NonZeroU32;
use anyhow::{anyhow, Result};
use std::rc::Rc;

const WINDOW_W: u32 = 640;
const WINDOW_H: u32 = 480; // 360;

#[derive(Default)]
struct App {
    windows: Vec<(Rc<Window>, softbuffer::Context<Rc<Window>>, softbuffer::Surface<Rc<Window>, Rc<Window>>)>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attr = Window::default_attributes()
            .with_title("Window")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(WINDOW_W, WINDOW_H));
        let window = Rc::new(event_loop.create_window(window_attr).unwrap());
        let context = softbuffer::Context::new(Rc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Rc::clone(&window)).unwrap();
        self.windows.push((window, context, surface));
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
                            if let Some((window, _, _)) = self.windows.iter_mut().find(|(window, _, _)| window.id() == id) {
                                window.set_fullscreen(if window.fullscreen().is_some() { None } else { Some(Fullscreen::Borderless(None)) });
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
                let (window, _context, surface) = self.windows.iter_mut().find(|(window, _, _)| window.id() == id).unwrap();
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

                    let (red, green, blue) = if x < 0 || x >= WINDOW_W as i32 || y < 0 || y >= WINDOW_H as i32 {
                        (0, 0, 0)
                    // } else if (x >= 100 && x <= 104) || (y >= 100 && y <= 104) {
                    } else if (x == 100) || (y == 100) {
                        (255, 255, 255)
                    } else {
                        (0, 80, 130)
                    };

                    buffer[index as usize] = blue | (green << 8) | (red << 16);
                }

                buffer.present().unwrap();
            }
            _ => {}
        }
    }
}

/*
            Event::NewEvents(start_cause) => {
                if false {
                    println!("\tNewEvents {:?}", start_cause);
                }
            }
            Event::DeviceEvent { device_id, event } => {
                if false {
                    println!("\tDeviceEvent: {:?}/{:?}", device_id, event);
                }
            }
            Event::UserEvent(_) => {
                println!("\tUserEvent");
            }
            Event::Suspended => {
                println!("\tSuspended");
            }
            Event::Resumed => {
                println!("\tResumed started={}", started);
                if !started {
                    started = true;

                    q.add(WindowCmd::CreateWindow(format!(
                        "window {}",
                        shared_state.count.fetch_add(1, Ordering::Relaxed) + 1
                    )));
                }
            }
            Event::AboutToWait => {
                if started && windows.len() == 0 {
                    // no more window ... exiting .. unless we have a system tray icon ???
                    target.exit();
                }
            }
            Event::LoopExiting => {
                println!("\tLoopExiting");
            }
            Event::MemoryWarning => {
                println!("\tMemoryWarning");
            }
        }

        while let Some(idx) = windows.iter().position(|w| w.exiting()) {
            windows.remove(idx);
        }

        for cmd in q.drain() {
            match cmd {
                WindowCmd::CreateWindow(title) => {
                    if let Ok(window) = WindowBuilder::new().with_title(title).build(target) {
                        windows.push(ZxWindow::new(shared_state.clone(), window));
                    }
                }
            }
        }
    };

    println!("event_loop.run called");
    event_loop.run(event_handler).map_err(|err| anyhow!(err))
}

fn main() -> Result<()> {
    run()
}
*/

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
