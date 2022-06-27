use crate::{Input, Mode};
use game::Game;
use log::info;
use network::{Client, Server};
use sdl2::keyboard::Keycode;

pub struct Gameplay {
    game: Option<Game>,
    server: Option<Server>,
    client: Client,
}

impl Gameplay {
    pub fn new(game: Option<Game>, client: Client, server: Option<Server>) -> Box<Self> {
        Box::new(Self {
            game,
            server,
            client,
        })
    }
}

impl Mode for Gameplay {
    fn update(&mut self, input: &Input) {
        if let Some(server) = self.server.as_mut() {
            for player in server.accept_players() {
                info!("Add player '{}' to game", player);
            }
            for player in server.lost_players() {
                info!("Remove player '{}' from game", player);
            }

            for request in server.requests() {
                // info!("Request [{}]: {:?}", request.player, request.request);
            }
        }

        for response in self.client.responses() {
            // info!("Response: {:?}", response);
        }

        if input.pressed(Keycode::P) {
            self.client.disconnect();
        }
    }
}
