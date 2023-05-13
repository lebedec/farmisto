use std::cell::RefCell;
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info};

use game::api::{Event, GameResponse, PlayerRequest};
use game::Game;
use network::TcpClient;

use crate::api::rpc::Procedure::{
    GetAgentInfo, GetAgentThinking, GetAgents, GetBehaviours, SaveBehaviours,
};
use crate::api::{handle_rpc, serve_web_socket};
use crate::{Behaviours, Nature};

pub struct AiThread {}

impl AiThread {
    pub fn spawn(
        mut client: TcpClient,
        behaviours: Arc<RwLock<Behaviours>>,
        knowledge: String,
    ) -> Self {
        let (call, mut calls) = channel();
        let (result, results) = channel();
        thread::spawn(move || serve_web_socket(call, results));
        thread::spawn(move || {
            let mut action_id = 0;
            let known = Game::load_knowledge(&knowledge);
            let mut nature = Nature {
                crops: vec![],
                creatures: vec![],
                creature_agents: vec![],
                tiles: Default::default(),
                containers: Default::default(),
                items: Default::default(),
                colonization_date: 0.0,
                farmers: Default::default(),
                feeding_map: vec![],
            };
            loop {
                let t = Instant::now();
                {
                    let events = handle_server_responses(&mut client);
                    nature.perceive(events, &known);
                    let behaviours = match behaviours.read() {
                        Ok(behaviours) => behaviours,
                        Err(_) => {
                            error!("Unable to read AI behaviours, assets thread terminated");
                            break;
                        }
                    };
                    nature.update();
                    for action in nature.react(&behaviours) {
                        info!("AI sends id={} {:?}", action_id, action);
                        client.send(PlayerRequest::Perform { action, action_id });
                        action_id += 1;
                    }
                }
                calls = match handle_rpc(&nature, calls, &result) {
                    Ok(calls) => calls,
                    Err(error) => {
                        error!("Unable to handle AI RPC, {error:?}");
                        return;
                    }
                };
                let elapsed = t.elapsed().as_secs_f32();

                // delay to simulate human reaction
                let delay = (0.25 - elapsed).max(0.0);
                thread::sleep(Duration::from_secs_f32(delay));
            }
        });

        Self {}
    }
}

fn handle_server_responses(client: &mut TcpClient) -> Vec<Event> {
    let responses: Vec<GameResponse> = client.responses().collect();
    let mut all_events = vec![];
    for response in responses {
        match response {
            GameResponse::Heartbeat => {}
            GameResponse::Events { events } => {
                all_events.extend(events);
            }
            GameResponse::Login { result } => {
                error!("Unexpected game login response result={:?}", result);
            }
            GameResponse::ActionError { action_id, error } => {
                error!("Action {} error response {:?}", action_id, error);
            }
            GameResponse::Trip { id } => client.send(PlayerRequest::Trip { id }),
        }
    }
    all_events
}
