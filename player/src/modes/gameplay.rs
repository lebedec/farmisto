use crate::{Input, Mode};
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::model::{TreeId, TreeKind};
use game::persistence::{Known, Shared, Storage};
use game::Game;
use log::{error, info};
use network::{Client, Configuration, Server};
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Gameplay {
    game: Option<GameThread>,
    client: Client,
    action_id: usize,
    storage: Storage,
    knowledge: KnowledgeBase,
    trees: HashMap<TreeId, TreeBehaviour>,
}

impl Gameplay {
    pub fn new(game: Option<GameThread>, client: Client) -> Box<Self> {
        Box::new(Self {
            game,
            client,
            action_id: 0,
            storage: Storage::open("./assets/database.sqlite").unwrap(),
            knowledge: KnowledgeBase::default(),
            trees: HashMap::new(),
        })
    }
}

impl Mode for Gameplay {
    fn update(&mut self, input: &Input) {
        self.knowledge.load(&self.storage);

        for response in self.client.responses() {
            match response {
                GameResponse::Heartbeat => {}
                GameResponse::Events { events } => {
                    for event in events {
                        match event {
                            Event::TreeAppeared {
                                id,
                                kind,
                                position,
                                growth,
                            } => {
                                let kind = self.knowledge.trees.get(kind).unwrap();
                                info!(
                                    "Appear tree {:?} kind='{}' at {:?} (g {})",
                                    id, kind.name, position, growth
                                );
                                self.trees.insert(id, TreeBehaviour { id, kind });
                            }
                            Event::TreeVanished { id } => {
                                info!("Vanish tree {:?}", id);
                                self.trees.remove(&id);
                            }
                        }
                    }
                }
                GameResponse::Login { result } => {
                    error!("Unexpected game login response result={:?}", result);
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

        // RENDER
        for _tree in self.trees.values() {}
    }
}

pub struct TreeBehaviour {
    id: TreeId,
    kind: Shared<TreeKind>,
}

#[derive(Default)]
pub struct KnowledgeBase {
    trees: Known<TreeKind>,
}

impl KnowledgeBase {
    pub fn load(&mut self, storage: &Storage) {
        self.trees.load(storage);
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
