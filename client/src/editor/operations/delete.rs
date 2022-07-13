use crate::editor::operations::Operation;
use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
use glam::{Vec2, Vec3};
use rusqlite::params;

pub struct Delete {}

impl Delete {
    pub fn new() -> Option<Box<dyn Operation>> {
        Some(Box::new(Self {}))
    }
}

impl Operation for Delete {
    fn handle(
        &mut self,
        input: &Input,
        assets: &mut Assets,
        storage: &Storage,
        gameplay: &mut Gameplay,
        selection: &mut Option<Selection>,
    ) -> bool {
        let current = match selection.as_ref() {
            None => return true,
            Some(selection) => selection,
        };
        match current {
            Selection::FarmlandProp { id, .. } => {
                assets
                    .storage
                    .connection()
                    .execute(
                        "delete from FarmlandAssetPropItem where id = ?",
                        params![id],
                    )
                    .unwrap();
                *selection = None;
            }
            Selection::Tree { .. } => {}
        }
        true
    }

    fn reset(&self) {
        todo!()
    }
}
