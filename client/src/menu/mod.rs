use glam::vec3;
use log::info;
use sdl2::keyboard::Keycode;

use crate::engine::rendering::{ButtonController, InputController};
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
    name_input: InputController,
    serve_button: ButtonController,
    join_button: ButtonController,
}

impl Menu {
    pub fn new(frame: &mut Frame) -> Box<Self> {
        Box::new(Self {
            join: None,
            host: None,
            name_input: frame.scene.instantiate_input(
                128,
                128,
                512,
                128,
                String::from("Host"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
                12,
            ),
            serve_button: frame.scene.instantiate_button(
                128 + 128 + 64,
                128,
                512,
                128,
                frame.translator.say("Button_serve"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
            ),
            join_button: frame.scene.instantiate_button(
                128 + 128 + 128 + 128,
                128,
                512,
                128,
                frame.translator.say("Button_join"),
                frame.assets.fonts_default.share(),
                frame.assets.sampler("default"),
                frame.assets.texture_white().share(),
            ),
        })
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.scene.look_at(vec3(0.0, 0.0, 0.0));

        frame.scene.render_input(&self.name_input);
        frame.scene.render_button(&self.serve_button);
        frame.scene.render_button(&self.join_button);
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

        self.name_input.update(&frame.input);
        self.serve_button
            .udpate(frame.input.mouse_position_raw(), frame.input.left_click());
        self.join_button
            .udpate(frame.input.mouse_position_raw(), frame.input.left_click());

        let player = self.name_input.get_value();

        if self.serve_button.clicked {
            info!("Host as {player}");
            self.host = Some(player.clone());
        }

        if self.join_button.clicked {
            if player != String::from("Host") {
                info!("Join as {player}");
                self.join = Some(player.clone())
            }
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
            let ai = AiThread::spawn(
                ai_client,
                ai_behaviours.share_data(),
                frame.config.save_file.clone(),
            );

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
