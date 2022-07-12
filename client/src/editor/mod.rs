use crate::gameplay::Gameplay;
use crate::{Assets, Input, MyRenderer};
use datamap::Storage;
use game::model::FarmlandId;
use glam::{Mat4, Vec2, Vec3};
use log::info;
use rusqlite::params;
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

#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    FarmlandProp {
        id: usize,
        farmland: FarmlandId,
        kind: String,
    },
}

trait Edit {
    fn handle(&mut self, input: &Input, assets: &mut Assets, storage: &Storage) -> bool;
    fn reset(&self);
}

struct Move {
    selection: Selection,
    lock: Vec3,
    mouse_origin: Vec2,
    position: Option<Vec3>,
    translation: Vec3,
}

impl Edit for Move {
    fn handle(&mut self, input: &Input, assets: &mut Assets, storage: &Storage) -> bool {
        if input.pressed(Keycode::X) {
            self.lock = Vec3::X;
        }

        if input.pressed(Keycode::Y) {
            self.lock = Vec3::Y;
        }

        if input.pressed(Keycode::Z) {
            self.lock = Vec3::Z;
        }

        let direction = self.lock.normalize_or_zero();
        let delta = Vec2::from(input.mouse_position().viewport) - self.mouse_origin;
        let delta = delta.x;

        self.translation = direction * delta;

        match &self.selection {
            Selection::FarmlandProp { farmland, id, kind } => {
                let mut asset = assets.farmlands.edit(&kind).unwrap();
                let prop = asset.props.iter_mut().find(|prop| &prop.id == id).unwrap();

                if self.position.is_none() {
                    self.position = Some(prop.position());
                }

                let position = self.position.unwrap();
                prop.position = (position + self.translation).into();

                if input.click() {
                    storage
                        .connection()
                        .execute(
                            "update FarmlandAssetPropItem set position = ? where id = ?",
                            params![datamap::to_json_value(prop.position.as_ref()), *id],
                        )
                        .unwrap();
                    return true;
                }
            }
        }

        false
    }

    fn reset(&self) {
        todo!()
    }
}

impl Gameplay {
    pub fn update_editor(&mut self, input: &Input, renderer: &mut MyRenderer, assets: &mut Assets) {
        let editor = self.editor.as_mut().unwrap();

        if input.pressed(Keycode::Tab) {
            editor.capture = !editor.capture;
        }

        if !editor.capture {
            return;
        }

        if let Some(edit) = editor.edit.as_mut() {
            if edit.handle(input, assets, &self.assets_storage) {
                editor.edit = None;
            }
        } else {
            if input.click() {
                let (_, pos) = self.camera.cast_ray(input.mouse_position());

                if let Some(pos) = pos {
                    let mut best = f32::INFINITY;
                    let mut best_position = Vec3::ZERO;
                    for farmland in self.farmlands.values() {
                        for prop in &farmland.asset.data.borrow().props {
                            let distance = prop.position().distance(pos);
                            if distance < best {
                                best = distance;
                                editor.selection = Some(Selection::FarmlandProp {
                                    id: prop.id,
                                    farmland: farmland.id,
                                    kind: farmland.kind.name.clone(),
                                })
                            }
                        }
                    }
                    info!("SELECTION: {:?}", editor.selection);
                }
            }

            self.handle_selection_command(input);
        }

        let editor = self.editor.as_mut().unwrap();
        match editor.selection.as_ref() {
            None => {}
            Some(Selection::FarmlandProp { farmland, id, .. }) => {
                let farmland = self.farmlands.get(farmland).unwrap();
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
        }
    }

    fn handle_selection_command(&mut self, input: &Input) {
        let editor = self.editor.as_mut().unwrap();
        let selection = match editor.selection.as_ref() {
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
                    editor.edit = Some(Box::new(Move {
                        selection: selection.clone(),
                        lock: Vec3::ZERO,
                        mouse_origin: Vec2::from(input.mouse_position().viewport),
                        position: None,
                        translation: Vec3::ZERO,
                    }));
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
