use crate::menu::Menu;
use crate::{Mode, SceneRenderer};

pub struct Intro {}

impl Intro {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Mode for Intro {
    fn transition(&self, _renderer: &mut SceneRenderer) -> Option<Box<dyn Mode>> {
        Some(Menu::new())
    }
}
