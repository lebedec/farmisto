use log::info;
use sdl2::keyboard::Keycode;

use ai::AiThread;
use network::{ClientMetrics, Configuration, TcpClient};
use server::LocalServerThread;

use crate::gameplay::{Gameplay, GameplayMetrics, Host};
use crate::monitoring::set_monitoring_context;
use crate::Frame;
use crate::Mode;

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
            set_monitoring_context(&player);
            let config = Configuration {
                host: player.clone(),
                port: frame.config.port,
                password: None,
                save_file: frame.config.save_file.clone(),
            };
            let server = LocalServerThread::spawn(config);
            let metrics = ClientMetrics::new(frame.metrics_registry).unwrap();
            let client =
                TcpClient::connect(&server.address, player.clone(), None, metrics).unwrap();

            let metrics = ClientMetrics::new_ai(frame.metrics_registry).unwrap();
            let ai_client =
                TcpClient::connect(&server.address, "<AI>".to_string(), None, metrics).unwrap();
            let ai_behaviours = frame.assets.behaviours("./assets/ai/nature.json");
            let ai = AiThread::spawn(ai_client, ai_behaviours.share_data());

            let metrics = GameplayMetrics::new(frame.metrics_registry).unwrap();
            let host = Host { server, ai };
            let gameplay = Gameplay::new(Some(host), client, frame, metrics);
            return Some(Box::new(gameplay));
        }
        if let Some(player) = self.join.as_ref() {
            set_monitoring_context(&player);
            let metrics = ClientMetrics::new(frame.metrics_registry).unwrap();
            let client =
                TcpClient::connect(&frame.config.host, player.to_string(), None, metrics).unwrap();
            let metrics = GameplayMetrics::new(frame.metrics_registry).unwrap();
            let gameplay = Gameplay::new(None, client, frame, metrics);
            return Some(Box::new(gameplay));
        }
        None
    }
}
