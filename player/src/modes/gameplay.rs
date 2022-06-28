use crate::{Input, Mode};
use game::api::{Action, GameResponse, PlayerRequest};
use game::Game;
use log::info;
use network::{Client, Configuration, Server};
use sdl2::keyboard::Keycode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Gameplay {
    game: Option<GameThread>,
    client: Client,
    action_id: usize,
}

impl Gameplay {
    pub fn new(game: Option<GameThread>, client: Client) -> Box<Self> {
        Box::new(Self {
            game,
            client,
            action_id: 0,
        })
    }
}

impl Mode for Gameplay {
    fn update(&mut self, input: &Input) {
        for response in self.client.responses() {
            match response {
                GameResponse::Heartbeat => {}
                _ => {
                    info!("Response: {:?}", response);
                }
            }
        }

        if input.pressed(Keycode::A) {
            self.action_id += 1;
            let action = Action::DoSomething;
            self.client.send(PlayerRequest::Perform {
                action,
                action_id: self.action_id,
            })
        }

        if input.pressed(Keycode::P) {
            self.client.disconnect();
        }

        if input.pressed(Keycode::O) {
            if let Some(thread) = self.game.as_mut() {
                thread.terminate();
            }
        }
    }
}

pub struct GameThread {
    running: Arc<AtomicBool>,
}

impl GameThread {
    pub fn spawn(config: Configuration) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_thread = running.clone();
        thread::spawn(move || {
            info!("Start game server thread");
            let mut game = Game::new();
            let mut server = Server::startup(config);
            let mut tick = Instant::now();
            while running_thread.load(Ordering::Relaxed) {
                for player in server.accept_players() {
                    info!("Add player '{}' to game", player);
                    let events = game.look_around();
                    server.send(player, GameResponse::Events { events })
                }
                for player in server.lost_players() {
                    info!("Remove player '{}' from game", player);
                }

                for request in server.requests() {
                    match request.request {
                        PlayerRequest::Heartbeat => {}
                        PlayerRequest::Perform { action, action_id } => {
                            let events = game.perform_action(action_id, action);
                            server.broadcast(GameResponse::Events { events });
                        }
                        _ => {
                            info!("Request [{}]: {:?}", request.player, request.request);
                        }
                    }
                }

                let time = tick.elapsed().as_secs_f32();
                tick = Instant::now();
                let events = game.update(time);
                if !events.is_empty() {
                    server.broadcast(GameResponse::Events { events });
                }

                thread::sleep(Duration::from_millis(20));
            }
            info!("Stop game server thread");
        });

        Self { running }
    }

    pub fn terminate(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }
}
