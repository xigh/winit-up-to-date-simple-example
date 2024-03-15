use std::sync::{atomic::Ordering, Arc};

use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::PhysicalKey,
    window::WindowBuilder,
};

use anyhow::{anyhow, Result};

mod window;
use window::ZxWindow;

mod cmd;
use cmd::CmdQueue;

use crate::{cmd::WindowCmd, window::ZxState};

pub fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;

    println!("event_loop created");

    let mut started = false;
    let mut windows: Vec<ZxWindow> = vec![];
    let shared_state = Arc::new(ZxState::default());

    let event_handler = move |event, target: &EventLoopWindowTarget<()>| {
        let q = CmdQueue::new();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                let Some(idx) = windows.iter().position(|w: &ZxWindow| w.id() == window_id) else {
                    // println!("\t\tcould not find window_id");
                    return;
                };

                match event {
                    WindowEvent::CloseRequested => {
                        println!("\tWindowEvent");
                        println!("\t\tCloseRequested");
                        windows.remove(idx);
                    }
                    WindowEvent::KeyboardInput {
                        event,
                        is_synthetic,
                        ..
                    } => match event.physical_key {
                        PhysicalKey::Code(code) => {
                            println!("\tWindowEvent");
                            println!(
                                "\t\tKeyboardInput {:?} - {:?} - {}",
                                code, event.state, is_synthetic
                            );
                            if !is_synthetic {
                                windows[idx].on_key_input(
                                    code,
                                    event.state == ElementState::Pressed,
                                    &q,
                                );
                            }
                        }
                        _ => {}
                    },
                    // WindowEvent::ActivationTokenDone { serial, token } => todo!(),
                    WindowEvent::Resized(new_size) => {
                        println!("\t\tResized {:?}", new_size);
                    }
                    // WindowEvent::Moved(_) => todo!(),
                    WindowEvent::Destroyed => {
                        println!("\t\tDestroyed");
                    }
                    // WindowEvent::DroppedFile(_) => todo!(),
                    // WindowEvent::HoveredFile(_) => todo!(),
                    // WindowEvent::HoveredFileCancelled => todo!(),
                    WindowEvent::Focused(focused) => {
                        println!("\t\tFocused {}", focused);
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        println!("\t\tModifiersChanged {:?}", modifiers);
                    }
                    // WindowEvent::Ime(_) => todo!(),
                    // WindowEvent::CursorMoved { device_id, position } => todo!(),
                    // WindowEvent::CursorEntered { device_id } => todo!(),
                    // WindowEvent::CursorLeft { device_id } => todo!(),
                    // WindowEvent::MouseWheel { device_id, delta, phase } => todo!(),
                    // WindowEvent::MouseInput { device_id, state, button } => todo!(),
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
                        println!("\t\tOccluded {}", occluded);
                    }
                    WindowEvent::RedrawRequested => {
                        println!("\t\tRedrawRequested")
                    }
                    _ => {}
                }
            }
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
