use crate::editor::operations::{Move, Operation};
use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input, Mode, MyRenderer};
use datamap::Storage;
use glam::{Mat4, Vec2, Vec3};
use log::info;
use sdl2::keyboard::Keycode;

pub struct Editor {
    pub selection: Option<Selection>,
    pub capture: bool,
    pub edit: Option<Box<dyn Operation>>,
    pub gameplay: Gameplay,
    pub storage: Storage,
}

impl Editor {
    fn handle_edit_operations(&mut self, input: &Input, assets: &mut Assets) {
        if input.pressed(Keycode::Tab) {
            self.capture = !self.capture;
        }

        if !self.capture {
            return;
        }

        if let Some(operation) = self.edit.as_mut() {
            if operation.handle(input, assets, &self.storage, &mut self.gameplay) {
                self.edit = None;
            }
            return;
        }

        if input.click() {
            let (_, pos) = self.gameplay.camera.cast_ray(input.mouse_position());

            if let Some(pos) = pos {
                let mut best = f32::INFINITY;
                let mut best_position = Vec3::ZERO;
                for farmland in self.gameplay.farmlands.values() {
                    for prop in &farmland.asset.data.borrow().props {
                        let distance = prop.position().distance(pos);
                        if distance < best {
                            best = distance;
                            self.selection = Some(Selection::FarmlandProp {
                                id: prop.id,
                                farmland: farmland.id,
                                kind: farmland.kind.name.clone(),
                            })
                        }
                    }
                }
                for tree in self.gameplay.trees.values() {
                    let distance = tree.position.distance(pos);
                    if distance < best {
                        best = distance;
                        self.selection = Some(Selection::Tree { id: tree.id })
                    }
                }
                info!("SELECTION: {:?}", self.selection);
            }
        }

        self.handle_selection_command(input);
    }

    fn handle_selection_command(&mut self, input: &Input) {
        let selection = match self.selection.as_ref() {
            None => return,
            Some(selection) => selection,
        };
        match selection {
            Selection::Tree { .. } | Selection::FarmlandProp { .. } => {
                if input.pressed(Keycode::D) {
                    // duplicate
                }
                if input.pressed(Keycode::X) {
                    // delete
                }
                if input.pressed(Keycode::G) {
                    self.edit = Move::new(
                        selection.clone(),
                        Vec2::from(input.mouse_position().viewport),
                    );
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

    fn render(&self, renderer: &mut MyRenderer) {
        match self.selection.as_ref() {
            None => {}
            Some(Selection::FarmlandProp { farmland, id, kind }) => {
                let farmland = self.gameplay.farmlands.get(farmland).unwrap();
                let data = farmland.asset.data.borrow();
                let props = data.props.iter().find(|p| p.id == *id).unwrap();
                let matrix = Mat4::from_translation(props.position.into())
                    * Mat4::from_scale(props.scale.into())
                    // todo: rework rotation
                    * Mat4::from_rotation_x(props.rotation[0].to_radians())
                    * Mat4::from_rotation_y(props.rotation[1].to_radians())
                    * Mat4::from_rotation_z(props.rotation[2].to_radians());
                renderer.bounds(matrix, props.asset.mesh().bounds());
            }
            Some(Selection::Tree { id }) => {
                let tree = self.gameplay.trees.get(id).unwrap();
                let matrix = Mat4::from_translation(tree.position);
                renderer.bounds(matrix, tree.asset.mesh().bounds());
            }
        }
    }
}

impl Mode for Editor {
    fn update(&mut self, input: &Input, renderer: &mut MyRenderer, assets: &mut Assets) {
        self.gameplay.knowledge.reload();
        self.gameplay.handle_server_responses(assets);
        self.handle_edit_operations(input, assets);
        self.gameplay.handle_user_input(input);
        self.gameplay.render(renderer);
        self.render(renderer);
    }
}
