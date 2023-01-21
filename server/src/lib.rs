use datamap::Storage;
use game::api::{ActionError, Event, GameResponse, PlayerRequest};
use game::model::UniverseSnapshot;
use game::Game;
use log::info;
use network::{Configuration, TcpServer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct LocalServerThread {
    pub running: Arc<AtomicBool>,
    pub address: String,
}

impl LocalServerThread {
    pub fn spawn(config: Configuration) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let (notify_started, started) = channel();
        let running_thread = running.clone();
        let port = config.port;
        let mut server = TcpServer::startup(config);
        let address = format!("{}:{}", server.address(), port);
        thread::spawn(move || {
            info!("Start game server thread");
            let storage = Storage::open("./assets/database.sqlite").unwrap();
            storage.setup_tracking().unwrap();
            let mut game = Game::new(storage);
            game.load_game_full();
            let mut tick = Instant::now();
            notify_started.send(true).unwrap();
            while running_thread.load(Ordering::Relaxed) {
                // game.hot_reload();
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
                            match game.perform_action(&request.player, action) {
                                Ok(events) => {
                                    server.broadcast(GameResponse::Events { events });
                                }
                                Err(error) => server.send(
                                    request.player,
                                    GameResponse::ActionError { action_id, error },
                                ),
                            }
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
        started.recv().unwrap();
        Self { running, address }
    }

    pub fn terminate(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }
}
