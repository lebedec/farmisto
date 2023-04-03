use crate::Frame;

pub trait Mode {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    #[allow(unused_variables)]
    fn update(&mut self, frame: &mut Frame) {}

    #[allow(unused_variables)]
    fn transition(&self, frame: &mut Frame) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}
