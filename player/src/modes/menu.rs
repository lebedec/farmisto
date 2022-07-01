use crate::engine::Input;
use crate::modes::{Gameplay, Mode};
use log::info;
use network::Client;
use sdl2::keyboard::Keycode;

pub struct Menu {
    join: Option<String>,
}

impl Menu {
    pub fn new() -> Box<Self> {
        Box::new(Self { join: None })
    }
}

impl Mode for Menu {
    fn update(&mut self, input: &Input) {
        if input.pressed(Keycode::E) {
            info!("Run editor mode")
        }

        if input.pressed(Keycode::J) {
            self.join = Some("Alice".to_string())
        }
    }

    fn transition(&self) -> Option<Box<dyn Mode>> {
        if let Some(player) = self.join.as_ref() {
            let client = Client::connect("127.0.0.1:8080", player.to_string(), None).unwrap();
            return Some(Gameplay::new(None, client));
        }
        None
    }
}
