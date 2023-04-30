use crate::engine::Frame;
use crate::menu::Menu;
use crate::Mode;

pub struct Intro {}

impl Intro {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Mode for Intro {
    fn transition(&self, frame: &mut Frame) -> Option<Box<dyn Mode>> {
        Some(Menu::new(frame))
    }
}
