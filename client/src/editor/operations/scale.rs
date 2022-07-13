use crate::editor::operations::Operation;
use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
use glam::{Vec2, Vec3};
use rusqlite::params;
use sdl2::keyboard::Keycode;

pub struct Scale {
    pub selection: Selection,
    pub lock: Vec3,
    pub mouse_origin: Vec2,
    pub scale: Option<Vec3>,
}

impl Scale {
    pub fn new(selection: Selection, mouse_origin: Vec2) -> Option<Box<dyn Operation>> {
        Some(Box::new(Self {
            selection,
            lock: Vec3::X + Vec3::Y + Vec3::Z,
            mouse_origin,
            scale: None,
        }))
    }
}

impl Operation for Scale {
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

        match &self.selection {
            Selection::FarmlandProp { farmland, id, kind } => {
                // self.farmlands

                let mut asset = assets.farmlands.edit(&kind).unwrap();
                let prop = asset.props.iter_mut().find(|prop| &prop.id == id).unwrap();

                if self.scale.is_none() {
                    self.scale = Some(prop.scale());
                }
                let scale = self.scale.unwrap();

                prop.scale = (scale + direction * delta).into();

                if input.click() {
                    assets
                        .storage
                        .connection()
                        .execute(
                            "update FarmlandAssetPropItem set scale = ? where id = ?",
                            params![datamap::to_json_value(prop.scale.as_ref()), *id],
                        )
                        .unwrap();
                    return true;
                }
            }
            Selection::Tree { id } => {}
        }

        false
    }

    fn reset(&self) {
        todo!()
    }
}
