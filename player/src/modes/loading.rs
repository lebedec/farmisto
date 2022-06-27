use crate::modes::{Gameplay, Menu};
use crate::Mode;
use game::Game;
use network::{Client, Configuration, Server};

pub struct Loading {
    is_editor: bool,
}

impl Loading {
    pub fn new(is_editor: bool) -> Box<Self> {
        Box::new(Self { is_editor })
    }
}

impl Mode for Loading {
    fn transition(&self) -> Option<Box<dyn Mode>> {
        if self.is_editor {
            // development mode startup
            let game = Some(Game::new());
            let player = "dev".to_string();
            let config = Configuration {
                host: player.clone(),
                password: None,
            };
            let server = Server::startup(config);
            let client = Client::connect("127.0.0.1:8080", player, None).unwrap();
            Some(Gameplay::new(game, client, Some(server)))
        } else {
            Some(Menu::new())
        }
    }
}
