use crate::editor::operations::Operation;
use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
use glam::{Vec2, Vec3};
use rusqlite::params;
use sdl2::keyboard::Keycode;

pub struct Move {
    pub selection: Selection,
    pub lock: Vec3,
    pub mouse_origin: Vec2,
    pub position: Option<Vec3>,
    pub translation: Vec3,
}

impl Move {
    pub fn new(selection: Selection, mouse_origin: Vec2) -> Option<Box<dyn Operation>> {
        Some(Box::new(Self {
            selection,
            lock: Vec3::X + Vec3::Z,
            mouse_origin,
            position: None,
            translation: Vec3::ZERO,
        }))
    }
}

impl Operation for Move {
    fn handle(
        &mut self,
        input: &Input,
        assets: &mut Assets,
        storage: &Storage,
        gameplay: &mut Gameplay,
        selection: &mut Option<Selection>,
    ) -> bool {
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

        let (ray, hit) = gameplay.camera.cast_ray(input.mouse_position());

        self.translation = hit.unwrap_or(Vec3::ZERO);

        match &self.selection {
            Selection::FarmlandProp { farmland, id, kind } => {
                // self.farmlands

                // let mut asset = assets.farmlands.edit(&kind).unwrap();
                // let prop = asset.props.iter_mut().find(|prop| &prop.id == id).unwrap();
                //
                // if self.position.is_none() {
                //     self.position = Some(prop.position());
                // }
                // let position = self.position.unwrap();
                //
                // // prop.position = (position + self.translation).into();
                // prop.position = self.translation.into();
                //
                // if input.click() {
                //     assets
                //         .storage
                //         .connection()
                //         .execute(
                //             "update FarmlandAssetPropItem set position = ? where id = ?",
                //             params![datamap::to_json_value(prop.position.as_ref()), *id],
                //         )
                //         .unwrap();
                //     return true;
                // }
            }
            Selection::Tree { id } => {
                let tree = gameplay.trees.get_mut(id).unwrap();

                if self.position.is_none() {
                    self.position = Some(tree.position);
                }
                let position = self.position.unwrap();

                // tree.position = position + self.translation;
                tree.position = self.translation;

                if input.click() {
                    let bp = [tree.position.x, tree.position.z];
                    let id: usize = tree.id.into();
                    storage
                        .connection()
                        .execute(
                            "update Barrier set position = ? where id = ?",
                            params![datamap::to_json_value(bp), id],
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
