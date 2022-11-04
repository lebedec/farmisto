use crate::{Assets, Frame};

pub trait Mode {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    #[allow(unused_variables)]
    fn start(&mut self, assets: &mut Assets) {}

    #[allow(unused_variables)]
    fn update(&mut self, context: Frame) {}

    fn transition(&self) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}
