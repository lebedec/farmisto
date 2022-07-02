use crate::engine::Input;
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

    fn start(&mut self) {}

    #[allow(unused_variables)]
    fn update(&mut self, input: &Input) {}

    fn transition(&self) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}
