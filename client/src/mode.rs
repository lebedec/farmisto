use crate::{Assets, Input, SceneRenderer};

pub trait Mode {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    #[allow(unused_variables)]
    fn start(&mut self, assets: &mut Assets) {}

    #[allow(unused_variables)]
    fn update(&mut self, input: &Input, renderer: &mut SceneRenderer, assets: &mut Assets) {}

    #[allow(unused_variables)]
    fn transition(&self, renderer: &mut SceneRenderer) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}
