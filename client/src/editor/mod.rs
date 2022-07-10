use crate::engine::FarmlandPrefab;
use crate::{Assets, Input, MyRenderer};
use sdl2::keyboard::Keycode;

pub struct Editor {
    pub lock_x: bool,
    pub lock_y: bool,
    pub lock_z: bool,
    pub selection: Option<Selection>,
    pub capture: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lock_x: false,
            lock_y: false,
            lock_z: false,
            selection: None,
            capture: false,
        }
    }
}

pub enum Selection {
    FarmlandProp { asset: FarmlandPrefab, prop: usize },
}

impl Editor {
    pub fn update(&mut self, input: &Input, _renderer: &mut MyRenderer, _assets: &mut Assets) {
        if input.pressed(Keycode::Tab) {
            self.capture = !self.capture;
        }

        if !self.capture {
            return;
        }

        self.handle_selection_command(input);
    }

    fn handle_selection_command(&mut self, input: &Input) {
        let selection = match self.selection.as_ref() {
            None => return,
            Some(selection) => selection,
        };
        match selection {
            Selection::FarmlandProp { .. } => {
                if input.pressed(Keycode::D) {
                    // duplicate
                }
                if input.pressed(Keycode::X) {
                    // delete
                }
                if input.pressed(Keycode::G) {
                    // go
                }
                if input.pressed(Keycode::R) {
                    // rotate
                }
                if input.pressed(Keycode::S) {
                    // scale
                }
            }
        }
    }
}
