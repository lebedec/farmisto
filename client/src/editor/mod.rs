use crate::{Assets, Input, MyRenderer};
use sdl2::keyboard::Keycode;

pub struct Editor {
    pub selection: Option<Selection>,
    pub capture: bool,
    pub edit: Option<Box<dyn Edit>>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            selection: None,
            capture: false,
            edit: None,
        }
    }
}

pub enum Selection {
    FarmlandProp { asset: usize, prop: usize },
}

trait Edit {
    fn handle(&mut self, input: &Input);
    fn reset(&self);
}

struct Move {
    lock_x: bool,
    lock_y: bool,
    lock_z: bool,
    origin: [f32; 2],
}

impl Edit for Move {
    fn handle(&mut self, input: &Input) {
        if input.pressed(Keycode::X) {
            self.lock_x = !self.lock_x;
        }

        if input.pressed(Keycode::Y) {
            self.lock_y = !self.lock_y;
        }

        if input.pressed(Keycode::Z) {
            self.lock_z = !self.lock_z;
        }

        // update translations set x = ?, y =? z =? where id = id
    }

    fn reset(&self) {
        todo!()
    }
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
                    //
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

    fn handle_move(&mut self, input: &Input) {}
}
