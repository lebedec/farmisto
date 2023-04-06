use game::math::VectorMath;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Input {
    pub time: f32,
    previous_mouse_position: [f32; 2],
    mouse_position: [f32; 2],
    mouse_viewport: [f32; 2],
    mouse_left_button_click: bool,
    mouse_right_button_click: bool,
    key_pressed: HashSet<Keycode>,
    key_down: HashSet<Keycode>,
    pub terminating: bool,
    pub window: [f32; 2],
    pub zoom: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Cursor {
    pub previous_position: [f32; 2],
    pub position: [f32; 2],
    pub viewport: [f32; 2],
    pub tile: [usize; 2],
}

impl Input {
    pub fn new(window: [u32; 2]) -> Self {
        Self {
            mouse_position: [0.0, 0.0],
            mouse_viewport: [0.0, 0.0],
            time: Default::default(),
            mouse_left_button_click: false,
            mouse_right_button_click: false,
            key_pressed: Default::default(),
            key_down: Default::default(),
            terminating: false,
            window: window.map(|value| value as f32),
            zoom: 1.0,
            previous_mouse_position: [0.0, 0.0],
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
                let x = x as f32 * self.zoom;
                let y = y as f32 * self.zoom;
                self.mouse_viewport = [
                    (2.0 * x) / self.window[0] - 1.0,
                    1.0 - (2.0 * y) / self.window[1],
                ];
                self.previous_mouse_position = self.mouse_position;
                self.mouse_position = [x, y];
            }
            Event::MouseButtonDown { .. } => {}
            Event::MouseButtonUp { mouse_btn, .. } => {
                if mouse_btn == MouseButton::Left {
                    self.mouse_left_button_click = true;
                }
                if mouse_btn == MouseButton::Right {
                    self.mouse_right_button_click = true;
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
        self.mouse_left_button_click = false;
        self.mouse_right_button_click = false;
        self.key_pressed.clear();
    }

    pub fn mouse_position(&self, camera_offset: [f32; 2], tile_size: f32) -> Cursor {
        let position = self.mouse_position.add(camera_offset).div(tile_size);
        let tile = position.to_tile();
        let viewport = self.mouse_viewport;
        Cursor {
            previous_position: self.previous_mouse_position.add(camera_offset).div(tile_size),
            position,
            viewport,
            tile,
        }
    }

    pub fn left_click(&self) -> bool {
        self.mouse_left_button_click
    }

    pub fn right_click(&self) -> bool {
        self.mouse_right_button_click
    }

    pub fn pressed(&self, key: Keycode) -> bool {
        self.key_pressed.contains(&key)
    }

    pub fn down(&self, key: Keycode) -> bool {
        self.key_down.contains(&key)
    }
}
