use crate::engine::Input;
use crate::gameplay::Gameplay;
use crate::{Mode, MyRenderer};
use log::info;
use network::TcpClient;
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
    fn update(&mut self, input: &Input, _renderer: &mut MyRenderer) {
        if input.pressed(Keycode::E) {
            info!("Run editor mode")
        }

        if input.pressed(Keycode::J) {
            self.join = Some("Alice".to_string())
        }
    }

    fn transition(&self, renderer: &mut MyRenderer) -> Option<Box<dyn Mode>> {
        if let Some(player) = self.join.as_ref() {
            let client = TcpClient::connect("127.0.0.1:8080", player.to_string(), None).unwrap();
            return Some(Gameplay::new(None, client, renderer.viewport));
        }
        None
    }
}
