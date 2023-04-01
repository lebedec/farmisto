use crate::gameplay::{Gameplay, Host};
use crate::Frame;
use crate::Mode;
use ai::AiThread;
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
    fn update(&mut self, frame: &mut Frame) {
        let input = &frame.input;

        if input.pressed(Keycode::E) {
            info!("Run editor mode")
        }

        if input.pressed(Keycode::S) {
            info!("Host as Alice");
            self.host = Some("Alice".to_string());
        }

        if input.pressed(Keycode::Num1) {
            info!("Join as Boris");
            self.join = Some("Boris".to_string())
        }

        if input.pressed(Keycode::Num2) {
            info!("Join as Carol");
            self.join = Some("Carol".to_string())
        }

        if input.pressed(Keycode::Num3) {
            info!("Join as David");
            self.join = Some("David".to_string())
        }
    }

    fn transition(&self, frame: &mut Frame) -> Option<Box<dyn Mode>> {
        if let Some(player) = self.host.as_ref() {
            let config = Configuration {
                host: player.clone(),
                port: frame.config.port,
                password: None,
            };
            let server = LocalServerThread::spawn(config);
            let client = TcpClient::connect(&server.address, player.clone(), None).unwrap();

            let ai_client = TcpClient::connect(&server.address, "<AI>".to_string(), None).unwrap();
            let ai_behaviours = frame.assets.behaviours("./assets/ai/nature.json");
            let ai = AiThread::spawn(ai_client, ai_behaviours.share_data());

            let host = Host { server, ai };
            let gameplay = Gameplay::new(Some(host), client, frame);
            return Some(Box::new(gameplay));
        }
        if let Some(player) = self.join.as_ref() {
            let client = TcpClient::connect(&frame.config.host, player.to_string(), None).unwrap();
            let gameplay = Gameplay::new(None, client, frame);
            return Some(Box::new(gameplay));
        }
        None
    }
}
