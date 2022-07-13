use crate::gameplay::Gameplay;
use crate::{Assets, Input};
use datamap::Storage;
pub use movement::*;

mod movement;

pub trait Operation {
    fn handle(
        &mut self,
        input: &Input,
        assets: &mut Assets,
        storage: &Storage,
        gameplay: &mut Gameplay,
    ) -> bool;
    fn reset(&self);
}
