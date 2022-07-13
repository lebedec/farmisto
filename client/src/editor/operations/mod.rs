use crate::editor::selection::Selection;
use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
pub use delete::*;
pub use duplicate::*;
pub use movement::*;
pub use rotation::*;
pub use scale::*;

mod delete;
mod duplicate;
mod movement;
mod rotation;
mod scale;

pub trait Operation {
    fn handle(
        &mut self,
        input: &Input,
        assets: &mut Assets,
        storage: &Storage,
        gameplay: &mut Gameplay,
        selection: &mut Option<Selection>,
    ) -> bool;
    fn reset(&self);
}
