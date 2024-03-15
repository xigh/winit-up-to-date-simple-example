# Simple (at least I tried) Rust winit example "0.29.15"

This simple example only create windows and manage windows list.

The [ZxWindow](https://github.com/xigh/winit-up-to-date-simple-example/blob/master/src/window/mod.rs#L27) structure is an abstration to a Window. You can put everything in here. For the moment, it only implements : 

```rust
pub fn on_key_input(&mut self, code: KeyCode, pressed: bool, queue: &Rc<CmdQueue>)
```

Where Queue allows the method to communicate with the event_loop, by example, in order to create new windows.

And the event_loop handles ZxWindow list. If this list is empty, the application exists.

# next steps : 

- [ ] add wgpu stuff 
- [ ] implements other events
- [ ] add fullscreen support
- [ ] add system tray icons
