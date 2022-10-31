use crate::editor::operations::Operation;
use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
use glam::{Vec2, Vec3};
use rusqlite::params;

pub struct Duplicate {
    pub selection: Selection,
    pub mouse_origin: Vec2,
}

impl Duplicate {
    pub fn new(selection: Selection, mouse_origin: Vec2) -> Option<Box<dyn Operation>> {
        Some(Box::new(Self {
            selection,
            mouse_origin,
        }))
    }
}

impl Operation for Duplicate {
    fn handle(
        &mut self,
        input: &Input,
        assets: &mut Assets,
        storage: &Storage,
        gameplay: &mut Gameplay,
        selection: &mut Option<Selection>,
    ) -> bool {
        /*
        let (ray, hit) = gameplay.camera.cast_ray(input.mouse_position());
        let position = hit.unwrap_or(Vec3::ZERO);

        match self.selection {
            Selection::FarmlandProp { id, .. } => {
                assets
                    .storage
                    .connection()
                    .execute(
                        "insert into FarmlandAssetPropItem (farmland, position, rotation, scale, asset) \
                         select farmland, ?, rotation, scale, asset \
                         from FarmlandAssetPropItem where id = ?",
                        params![datamap::to_json_value(position.as_ref()), id],
                    )
                    .unwrap();
            }
            Selection::Tree { .. } => {}
        }*/
        true
    }

    fn reset(&self) {
        todo!()
    }
}
