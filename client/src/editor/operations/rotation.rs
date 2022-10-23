use crate::editor::operations::Operation;
use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
use glam::{Vec2, Vec3};
use rusqlite::params;
use sdl2::keyboard::Keycode;

pub struct Rotation {
    pub selection: Selection,
    pub lock: Vec3,
    pub mouse_origin: Vec2,
    pub rotation: Option<Vec3>,
    pub angle: f32,
}

impl Rotation {
    pub fn new(selection: Selection, mouse_origin: Vec2) -> Option<Box<dyn Operation>> {
        Some(Box::new(Self {
            selection,
            lock: Vec3::Y,
            mouse_origin,
            rotation: None,
            angle: 0.0,
        }))
    }
}

impl Operation for Rotation {
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

        self.angle = delta * 180.0;

        match &self.selection {
            Selection::FarmlandProp { id, kind, .. } => {
                // self.farmlands

                // let mut asset = assets.farmlands.edit(&kind).unwrap();
                // let prop = asset.props.iter_mut().find(|prop| &prop.id == id).unwrap();
                //
                // if self.rotation.is_none() {
                //     self.rotation = Some(prop.rotation());
                // }
                // let rotation = self.rotation.unwrap();
                //
                // prop.rotation = (rotation + self.angle * direction).into();
                //
                // if input.click() {
                //     assets
                //         .storage
                //         .connection()
                //         .execute(
                //             "update FarmlandAssetPropItem set rotation = ? where id = ?",
                //             params![datamap::to_json_value(prop.rotation.as_ref()), *id],
                //         )
                //         .unwrap();
                //     return true;
                // }
            }
            Selection::Tree { .. } => {}
        }

        false
    }

    fn reset(&self) {
        todo!()
    }
}
