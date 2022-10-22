use crate::engine::Input;
use crate::gameplay::Gameplay;
use crate::{Assets, Mode, SceneRenderer};
use log::info;
use network::{Configuration, TcpClient};
use sdl2::keyboard::Keycode;
use server::LocalServerThread;

pub struct Menu {
    host: Option<String>,
    join: Option<String>,
}

impl Menu {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            join: None,
            host: None,
        })
    }
}

impl Mode for Menu {
    fn update(&mut self, input: &Input, _renderer: &mut SceneRenderer, _assets: &mut Assets) {
        if input.pressed(Keycode::E) {
            info!("Run editor mode")
        }

        if input.pressed(Keycode::S) {
            info!("Host as Boris");
            self.host = Some("Boris".to_string());
        }

        if input.pressed(Keycode::A) {
            info!("Join as Alice");
            self.join = Some("Alice".to_string())
        }
    }

    fn transition(&self, renderer: &mut SceneRenderer) -> Option<Box<dyn Mode>> {
        if let Some(player) = self.host.as_ref() {
            let config = Configuration {
                host: player.clone(),
                port: 8080,
                password: None,
            };
            let server = LocalServerThread::spawn(config);
            let client = TcpClient::connect(&server.address, player.clone(), None).unwrap();
            let gameplay = Gameplay::new(Some(server), client, renderer.viewport);
            return Some(Box::new(gameplay));
        }
        if let Some(player) = self.join.as_ref() {
            let client = TcpClient::connect("127.0.0.1:8080", player.to_string(), None).unwrap();
            return Some(Box::new(Gameplay::new(None, client, renderer.viewport)));
        }
        None
    }
}
