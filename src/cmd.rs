use std::{cell::RefCell, rc::Rc};

pub enum WindowCmd {
    CreateWindow(String),
}

pub struct CmdQueue {
    commands: RefCell<Vec<WindowCmd>>,
}

impl CmdQueue {
    pub fn new() -> Rc<Self> {
        Rc::new(CmdQueue {
            commands: RefCell::new(Vec::new()),
        })
    }

    pub fn add(&self, command: WindowCmd) {
        self.commands.borrow_mut().push(command);
    }

    pub fn drain(&self) -> Vec<WindowCmd> {
        self.commands.borrow_mut().drain(..).collect()
    }
}
