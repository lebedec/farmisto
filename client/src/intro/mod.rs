use crate::editor::Editor;
use crate::gameplay::Gameplay;
use crate::menu::Menu;
use crate::{Mode, SceneRenderer};
use datamap::Storage;
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
    fn transition(&self, renderer: &mut SceneRenderer) -> Option<Box<dyn Mode>> {
        if self.is_editor {
            // development mode startup
            let player = "dev".to_string();
            let config = Configuration {
                host: player.clone(),
                password: None,
            };
            let server = LocalServerThread::spawn(config);
            thread::sleep(Duration::from_millis(10));
            let client = TcpClient::connect("127.0.0.1:8080", player, None).unwrap();
            let gameplay = Gameplay::new(Some(server), client, renderer.viewport);

            if self.is_editor {
                Some(Box::new(Editor {
                    selection: None,
                    active: false,
                    operation: None,
                    gameplay,
                    storage: Storage::open("./assets/database.sqlite").unwrap(),
                }))
            } else {
                Some(Box::new(gameplay))
            }
        } else {
            Some(Menu::new())
        }
    }
}
