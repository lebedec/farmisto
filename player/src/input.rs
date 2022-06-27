use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Input {
    pub time: f32,
    mouse_position: [f32; 2],
    mouse_button_click: bool,
    key_pressed: HashSet<Keycode>,
    key_down: HashSet<Keycode>,
    pub terminating: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Cursor {
    pub position: [f32; 2],
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_position: [0.0, 0.0],
            time: Default::default(),
            mouse_button_click: false,
            key_pressed: Default::default(),
            key_down: Default::default(),
            terminating: false,
        }
    }

    pub fn handle(&mut self, event: Event) {
        match event {
            Event::Quit { .. } => {
                self.terminating = true;
            }
            Event::AppTerminating { .. } => {}
            Event::AppLowMemory { .. } => {}
            Event::AppWillEnterBackground { .. } => {}
            Event::AppDidEnterBackground { .. } => {}
            Event::AppWillEnterForeground { .. } => {}
            Event::AppDidEnterForeground { .. } => {}
            Event::Display { .. } => {}
            Event::Window { .. } => {}
            Event::KeyDown { keycode, .. } => {
                if let Some(keycode) = keycode {
                    self.key_down.insert(keycode);
                }
            }
            Event::KeyUp { keycode, .. } => {
                if let Some(keycode) = keycode {
                    self.key_down.remove(&keycode);
                    self.key_pressed.insert(keycode);
                }
            }
            Event::TextEditing { .. } => {}
            Event::TextInput { .. } => {}
            Event::MouseMotion { x, y, .. } => {
                self.mouse_position = [x as f32, y as f32];
            }
            Event::MouseButtonDown { .. } => {}
            Event::MouseButtonUp { mouse_btn, .. } => {
                if mouse_btn == MouseButton::Left {
                    self.mouse_button_click = true;
                }
            }
            Event::MouseWheel { .. } => {}
            Event::JoyAxisMotion { .. } => {}
            Event::JoyBallMotion { .. } => {}
            Event::JoyHatMotion { .. } => {}
            Event::JoyButtonDown { .. } => {}
            Event::JoyButtonUp { .. } => {}
            Event::JoyDeviceAdded { .. } => {}
            Event::JoyDeviceRemoved { .. } => {}
            Event::ControllerAxisMotion { .. } => {}
            Event::ControllerButtonDown { .. } => {}
            Event::ControllerButtonUp { .. } => {}
            Event::ControllerDeviceAdded { .. } => {}
            Event::ControllerDeviceRemoved { .. } => {}
            Event::ControllerDeviceRemapped { .. } => {}
            Event::FingerDown { .. } => {}
            Event::FingerUp { .. } => {}
            Event::FingerMotion { .. } => {}
            Event::DollarGesture { .. } => {}
            Event::DollarRecord { .. } => {}
            Event::MultiGesture { .. } => {}
            Event::ClipboardUpdate { .. } => {}
            Event::DropFile { .. } => {}
            Event::DropText { .. } => {}
            Event::DropBegin { .. } => {}
            Event::DropComplete { .. } => {}
            Event::AudioDeviceAdded { .. } => {}
            Event::AudioDeviceRemoved { .. } => {}
            Event::RenderTargetsReset { .. } => {}
            Event::RenderDeviceReset { .. } => {}
            Event::User { .. } => {}
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.mouse_button_click = false;
        self.key_pressed.clear();
    }

    pub fn mouse_position(&self) -> Cursor {
        Cursor {
            position: self.mouse_position,
        }
    }

    pub fn click(&self) -> bool {
        self.mouse_button_click
    }

    pub fn pressed(&self, key: Keycode) -> bool {
        self.key_pressed.contains(&key)
    }

    pub fn down(&self, key: Keycode) -> bool {
        self.key_down.contains(&key)
    }
}
