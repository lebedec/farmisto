use crate::{Input, Mode};
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::model::{TreeId, TreeKind};
use game::persistence::{Known, Shared, Storage};
use hosting::GameHostingThread;
use log::{error, info};
use network::Client;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

pub struct Gameplay {
    game: Option<GameHostingThread>,
    client: Client,
    action_id: usize,
    storage: Storage,
    knowledge: KnowledgeBase,
    trees: HashMap<TreeId, TreeBehaviour>,
}

impl Gameplay {
    pub fn new(game: Option<GameHostingThread>, client: Client) -> Box<Self> {
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
                            Event::TreeVanished(id) => {
                                info!("Vanish tree {:?}", id);
                                self.trees.remove(&id);
                            }
                            Event::TreeUpdated { id } => {
                                info!("Update tree {:?} [not implemented yet]", id);
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
