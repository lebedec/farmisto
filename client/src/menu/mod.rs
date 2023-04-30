use glam::vec3;
use log::info;
use sdl2::keyboard::Keycode;

use crate::engine::rendering::{ButtonController, TextController};
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
    serve_button: ButtonController,
    join_button_boris: ButtonController,
    join_button_carol: ButtonController,
    join_button_david: ButtonController,
}

impl Menu {
    pub fn new(frame: &mut Frame) -> Box<Self> {
        Box::new(Self {
            join: None,
            host: None,
            serve_button: frame.scene.instantiate_button(
                128,
                128,
                512,
                128,
                frame.translator.say("Button_serve"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
            ),
            join_button_boris: frame.scene.instantiate_button(
                128 + 128 + 64,
                128,
                512,
                128,
                frame.translator.say("Button_join_boris"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
            ),
            join_button_carol: frame.scene.instantiate_button(
                128 + 128 + 128 + 128,
                128,
                512,
                128,
                frame.translator.say("Button_join_carol"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
            ),
            join_button_david: frame.scene.instantiate_button(
                128 + 128 + 128 + 128 + 128 + 64,
                128,
                512,
                128,
                frame.translator.say("Button_join_david"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
            ),
        })
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.scene.look_at(vec3(0.0, 0.0, 0.0));

        frame.scene.render_button(&self.serve_button);
        frame.scene.render_button(&self.join_button_boris);
        frame.scene.render_button(&self.join_button_carol);
        frame.scene.render_button(&self.join_button_david);
    }
}

impl Mode for Menu {
    fn update(&mut self, frame: &mut Frame) {
        let input = &frame.input;

        if input.pressed(Keycode::P) {
            frame
                .assets
                .texture("assets/texture/building-construction.png");
            frame
                .assets
                .texture("assets/texture/building-deconstruction.png");
        }

        self.serve_button
            .udpate(frame.input.mouse_position_raw(), frame.input.left_click());
        self.join_button_boris
            .udpate(frame.input.mouse_position_raw(), frame.input.left_click());
        self.join_button_carol
            .udpate(frame.input.mouse_position_raw(), frame.input.left_click());
        self.join_button_david
            .udpate(frame.input.mouse_position_raw(), frame.input.left_click());

        if self.serve_button.clicked {
            info!("Host as Alice");
            self.host = Some("Alice".to_string());
        }

        if self.join_button_boris.clicked {
            info!("Join as Boris");
            self.join = Some("Boris".to_string())
        }

        if self.join_button_carol.clicked {
            info!("Join as Carol");
            self.join = Some("Carol".to_string())
        }

        if self.join_button_david.clicked {
            info!("Join as David");
            self.join = Some("David".to_string())
        }

        self.render(frame);
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
