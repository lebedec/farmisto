use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use log::info;

use datamap::Storage;
use game::api::{GameResponse, PlayerRequest};
use game::model::UniverseSnapshot;
use game::Game;
use lazy_static::lazy_static;
use network::{Configuration, TcpServer};

lazy_static! {
    static ref HOST_FRAMES_TOTAL: prometheus::IntCounter =
        prometheus::register_int_counter!("host_frames_total", "host_frames_total").unwrap();
}

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
        thread::Builder::new()
            .name("game".into())
            .spawn(move || {
                info!("Start game server thread");
                let storage = Storage::open("./assets/database.sqlite").unwrap();
                storage.setup_tracking().unwrap();
                let mut game = Game::new(storage);
                game.load_game_full();
                let mut tick = Instant::now();
                notify_started.send(true).unwrap();

                let m_fps_time = 0.0;
                let m_fps = 0;

                while running_thread.load(Ordering::Relaxed) {
                    HOST_FRAMES_TOTAL.inc();

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
                                    Err(response) => server.send(
                                        request.player,
                                        GameResponse::ActionError {
                                            action_id,
                                            response,
                                        },
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
            })
            .unwrap();
        started.recv().unwrap();
        Self { running, address }
    }

    pub fn terminate(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }
}
