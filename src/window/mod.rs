use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use winit::{
    keyboard::KeyCode,
    window::{Window, WindowId},
};

use crate::{CmdQueue, WindowCmd};

#[derive(Default)]
pub struct ZxState {
    pub count: AtomicUsize,
}

pub struct ZxWindow {
    shared_state: Arc<ZxState>,
    window: Window,
    is_exiting: bool,
}

impl ZxWindow {
    pub fn new(shared_state: Arc<ZxState>, window: Window) -> Self {
        Self {
            shared_state,
            window,
            is_exiting: false,
        }
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn exiting(&self) -> bool {
        self.is_exiting
    }

    pub fn on_key_input(&mut self, code: KeyCode, pressed: bool, queue: &Rc<CmdQueue>) {
        println!(
            "# on_key {:?} - {}",
            code,
            if pressed { "pressed" } else { "released " }
        );
        if !pressed {
            return;
        }

        match code {
            // exit on `Escape`
            KeyCode::Escape => self.is_exiting = true,

            // create new window on `KeyN`
            KeyCode::KeyN => queue.add(WindowCmd::CreateWindow(format!(
                "Window {}",
                self.shared_state.count.fetch_add(1, Ordering::Relaxed) + 1
            ))),
            _ => {}
        }
    }
}
