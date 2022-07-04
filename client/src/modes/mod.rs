use crate::engine::Input;
use crate::{AssetManager, MyRenderer};
pub use gameplay::*;
pub use loading::*;
pub use menu::*;

mod gameplay;
mod loading;
mod menu;

pub trait Mode {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    #[allow(unused_variables)]
    fn start(&mut self, manager: &mut AssetManager) {}

    #[allow(unused_variables)]
    fn update(&mut self, input: &Input, renderer: &mut MyRenderer) {}

    fn transition(&self) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}
