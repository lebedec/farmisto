use game::api::{GameResponse, PlayerRequest};
use game::model::UniverseSnapshot;
use game::Game;
use log::info;
use network::{Configuration, Server};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct GameHostingThread {
    running: Arc<AtomicBool>,
}

impl GameHostingThread {
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
                    let events = game.look_around(UniverseSnapshot::whole());
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
