use crate::{Assets, Input, MyRenderer};

pub trait Mode {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    #[allow(unused_variables)]
    fn start(&mut self, manager: &mut Assets) {}

    #[allow(unused_variables)]
    fn update(&mut self, input: &Input, renderer: &mut MyRenderer, assets: &mut Assets) {}

    #[allow(unused_variables)]
    fn transition(&self, renderer: &mut MyRenderer) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}
