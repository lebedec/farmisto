use crate::gameplay::Gameplay;
use crate::menu::Menu;
use crate::{Mode, MyRenderer};
use network::{Configuration, TcpClient};
use server::LocalServerThread;
use std::thread;
use std::time::Duration;

pub struct Intro {
    is_editor: bool,
}

impl Intro {
    pub fn new(is_editor: bool) -> Box<Self> {
        Box::new(Self { is_editor })
    }
}

impl Mode for Intro {
    fn transition(&self, renderer: &mut MyRenderer) -> Option<Box<dyn Mode>> {
        if self.is_editor {
            // development mode startup
            let player = "dev".to_string();
            let config = Configuration {
                host: player.clone(),
                password: None,
            };
            let server = LocalServerThread::spawn(config);
            // await server start
            thread::sleep(Duration::from_millis(100));
            let client = TcpClient::connect("127.0.0.1:8080", player, None).unwrap();
            Some(Gameplay::new(Some(server), client, renderer.viewport))
        } else {
            Some(Menu::new())
        }
    }
}
