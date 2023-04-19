use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info};
use rand::{thread_rng, Rng};

use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::VectorMath;
use game::model::{Creature, Crop, Universe};
use game::physics::{Physics, SpaceId};
use network::TcpClient;

use crate::api::serve_web_socket;
use crate::decision_making::{
    consider, make_decision, Behaviour, Choice, Decision, DecisionRef, Thinking,
};

mod api;
mod decision_making;

pub struct AiThread {}

impl AiThread {
    pub fn spawn(mut client: TcpClient, behaviours: Arc<RwLock<Behaviours>>) -> Self {
        let nature = Nature {
            crops: vec![],
            creatures: vec![],
            creature_agents: vec![],
            tiles: Default::default(),
        };
        let nature_lock = Arc::new(RwLock::new(nature));
        let nature_read_access = nature_lock.clone();
        thread::spawn(move || serve_web_socket(nature_read_access));
        thread::spawn(move || {
            let mut action_id = 0;
            loop {
                let t = Instant::now();
                {
                    let mut nature = nature_lock.write().unwrap();
                    let events = handle_server_responses(&mut client);
                    nature.perceive(events);
                    for action in nature.react(&behaviours.read().unwrap()) {
                        info!("AI sends id={} {:?}", action_id, action);
                        client.send(PlayerRequest::Perform { action, action_id });
                        action_id += 1;
                    }
                }
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
            GameResponse::ActionError {
                action_id,
                response,
            } => {
                error!("Action {} error response {:?}", action_id, response);
            }
        }
    }
    all_events
}

pub struct CropView {
    entity: Crop,
    growth: f32,
    position: [f32; 2],
}

pub struct FarmerView {}

pub struct CreatureView {
    _entity: Creature,
}

pub struct InvaserView {
    _threat: f32,
}

pub struct CreatureAgent {
    creature: Creature,
    space: SpaceId,
    hunger: f32,
    _mindset: Vec<String>,
    history: HashMap<DecisionRef, Instant>,
    thinking: Thinking,
    position: [f32; 2],
    radius: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentRef {
    id: usize,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CreatureCropAction {
    Nothing,
    EatCrop,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CreatureCropInput {
    Constant,
    Hunger,
    CropDistance,
    CropNutritionValue,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CreatureGroundAction {
    MoveCreature,
    Delay { min: f32, max: f32 },
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CreatureGroundInput {
    Constant,
    Random,
    Cooldown(f32, f32),
    Distance,
}

type CreatureCropDecision = Decision<CreatureCropInput, CreatureCropAction>;
type CreatureGroundDecision = Decision<CreatureGroundInput, CreatureGroundAction>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Behaviours {
    creatures: Vec<CreatureBehaviourSet>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum CreatureBehaviourSet {
    Crop {
        name: String,
        behaviours: Vec<Behaviour<CreatureCropDecision>>,
    },
    Ground {
        name: String,
        behaviours: Vec<Behaviour<CreatureGroundDecision>>,
    },
}

#[derive(Default, Clone, Copy)]
struct Tile {
    has_hole: bool,
    has_barrier: bool,
}

pub enum Tuning {
    Delay { behaviour: usize },
}

impl Into<Choice<Action, Tuning>> for Tuning {
    fn into(self) -> Choice<Action, Tuning> {
        Choice::Tuning(self)
    }
}

impl Into<Choice<Action, Tuning>> for Action {
    fn into(self) -> Choice<Action, Tuning> {
        Choice::Action(self)
    }
}

pub struct Nature {
    // game view
    crops: Vec<CropView>,
    creatures: Vec<CreatureView>,
    // agents
    creature_agents: Vec<CreatureAgent>,
    tiles: HashMap<SpaceId, Vec<Vec<Tile>>>,
}

impl Nature {
    pub fn perceive(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::UniverseStream(events) => {
                    for event in events {
                        match event {
                            Universe::ActivityChanged { .. } => {}
                            Universe::TreeAppeared { .. } => {}
                            Universe::TreeVanished(_) => {}
                            Universe::FarmlandAppeared {
                                farmland, holes, ..
                            } => {
                                let mut tiles = vec![vec![Tile::default(); 128]; 128];
                                for y in 0..holes.len() {
                                    for x in 0..holes.len() {
                                        tiles[y][x].has_hole = holes[y][x] == 1;
                                    }
                                }
                                self.tiles.insert(farmland.space, tiles);
                            }
                            Universe::FarmlandVanished(_) => {}
                            Universe::FarmerAppeared { .. } => {}
                            Universe::FarmerVanished(_) => {}
                            Universe::StackAppeared { .. } => {}
                            Universe::StackVanished(_) => {}
                            Universe::CropAppeared {
                                entity,
                                growth,
                                position,
                                ..
                            } => self.crops.push(CropView {
                                entity,
                                growth,
                                position,
                            }),
                            Universe::CropVanished(_) => {}
                            Universe::ConstructionAppeared { .. } => {}
                            Universe::ConstructionVanished { .. } => {}
                            Universe::EquipmentAppeared { .. } => {}
                            Universe::EquipmentVanished(_) => {}
                            Universe::ItemsAppeared { .. } => {}
                            Universe::CreatureAppeared {
                                entity,
                                position,
                                hunger,
                                space,
                                ..
                            } => {
                                self.creatures.push(CreatureView { _entity: entity });
                                self.creature_agents.push(CreatureAgent {
                                    creature: entity,
                                    space,
                                    hunger,
                                    _mindset: vec![],
                                    history: Default::default(),
                                    thinking: Thinking::default(),
                                    position,
                                    radius: 5,
                                })
                            }
                            Universe::CreatureEats { .. } => {}
                            Universe::CreatureVanished(_) => {}
                            Universe::AssemblyAppeared { .. } => {}
                            Universe::AssemblyUpdated { .. } => {}
                            Universe::AssemblyVanished(_) => {}
                            Universe::DoorAppeared { .. } => {}
                            Universe::DoorVanished(_) => {}
                            Universe::DoorChanged { .. } => {}
                            Universe::CementerAppeared { .. } => {}
                            Universe::CementerVanished(_) => {}
                        }
                    }
                }
                Event::PhysicsStream(events) => {
                    for event in events {
                        match event {
                            Physics::BodyPositionChanged { id, position, .. } => {
                                for agent in self.creature_agents.iter_mut() {
                                    if agent.creature.body == id {
                                        agent.position = position;
                                        break;
                                    }
                                }
                            }
                            Physics::BarrierCreated {
                                space, position, ..
                            } => {
                                let tiles = self.tiles.get_mut(&space).unwrap();
                                let [x, y] = position.to_tile();
                                // TODO: barrier bounds
                                tiles[y][x].has_barrier = true;
                            }
                            Physics::BarrierChanged { .. } => {}
                            Physics::BarrierDestroyed {
                                position, space, ..
                            } => {
                                let tiles = self.tiles.get_mut(&space).unwrap();
                                let [x, y] = position.to_tile();
                                // TODO: multiple barriers on same tile
                                tiles[y][x].has_barrier = false;
                            }
                            Physics::SpaceUpdated { id, holes } => {
                                let tiles = self.tiles.get_mut(&id).unwrap();
                                for y in 0..holes.len() {
                                    for x in 0..holes.len() {
                                        tiles[y][x].has_hole = holes[y][x] == 1;
                                    }
                                }
                            }
                        }
                    }
                }
                Event::BuildingStream(_) => {}
                Event::InventoryStream(_) => {}
                Event::PlantingStream(_) => {}
                Event::RaisingStream(_) => {}
                Event::AssemblingStream(_) => {}
                Event::WorkingStream(_) => {}
                Event::LandscapingStream(_) => {}
                Event::TimingStream(_) => {}
            }
        }
    }

    fn get_empty_tiles(
        game_tiles: &Vec<Vec<Tile>>,
        center: [usize; 2],
        radius: usize,
    ) -> Vec<[usize; 2]> {
        let mut tiles = vec![];
        let mut map = vec![vec![0; 128]; 128];
        let mut frontier = vec![center];
        let mut wave = 1;
        loop {
            let mut new_wave = vec![];
            for current in frontier {
                let [cx, cy] = current;
                map[cy][cx] = wave;
                tiles.push(current);
                let cx = cx as isize;
                let cy = cy as isize;
                let steps: [[isize; 2]; 4] =
                    [[cx, cy - 1], [cx - 1, cy], [cx + 1, cy], [cx, cy + 1]];
                for next in steps {
                    let [nx, ny] = next;
                    if nx >= 0 && nx < 128 && ny >= 0 && ny < 128 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        let tile = &game_tiles[ny][nx];
                        let not_empty = tile.has_barrier || tile.has_hole;
                        if not_empty {
                            // mark blocked tiles
                            map[ny][nx] = 1;
                        } else if map[ny][nx] == 0 {
                            map[ny][nx] = wave;
                            new_wave.push([nx as usize, ny as usize]);
                        }
                    }
                }
            }
            wave += 1;
            if wave == radius + 2 || new_wave.len() == 0 {
                break;
            }
            frontier = new_wave;
        }
        tiles
    }

    pub fn react(&mut self, behaviours: &Behaviours) -> Vec<Action> {
        let mut actions = vec![];
        for agent in self.creature_agents.iter_mut() {
            let (choice, decision, thinking) = make_decision(
                &behaviours.creatures,
                |set_index, set, thinking| match set {
                    CreatureBehaviourSet::Crop { behaviours, .. } => {
                        let (b, t, d, scores) = consider(
                            set_index,
                            &behaviours,
                            &self.crops,
                            |_, input, crop| match input {
                                CreatureCropInput::Hunger => agent.hunger,
                                CreatureCropInput::CropDistance => {
                                    crop.position.distance(agent.position) / 10.0
                                }
                                CreatureCropInput::CropNutritionValue => crop.growth / 5.0,
                                CreatureCropInput::Constant => 0.0,
                            },
                            thinking,
                        );
                        let action = behaviours[b].decisions[d].action;
                        let crop = self.crops[t].entity;
                        let choice = match action {
                            CreatureCropAction::EatCrop => Action::EatCrop {
                                crop,
                                creature: agent.creature,
                            }
                            .into(),
                            CreatureCropAction::Nothing => Choice::Nothing,
                        };
                        (scores, b, d, choice)
                    }
                    CreatureBehaviourSet::Ground { behaviours, .. } => {
                        let game_tiles = self.tiles.get(&agent.space).unwrap();
                        let targets = Self::get_empty_tiles(
                            game_tiles,
                            agent.position.to_tile(),
                            agent.radius,
                        );
                        let (b, t, d, scores) = consider(
                            set_index,
                            &behaviours,
                            &targets,
                            |decision, input, tile| match input {
                                CreatureGroundInput::Constant => 1.0,
                                CreatureGroundInput::Random => thread_rng().gen_range(0.0..=1.0),
                                CreatureGroundInput::Cooldown(start, end) => {
                                    let delta = end - start;
                                    let elapsed = agent
                                        .history
                                        .get(&decision)
                                        .map(|time| time.elapsed().as_secs_f32())
                                        .unwrap_or(end);
                                    (elapsed - start) / delta
                                }
                                CreatureGroundInput::Distance => {
                                    agent.position.distance(position_of(*tile)) / 10.0
                                }
                            },
                            thinking,
                        );
                        let action = behaviours[b].decisions[d].action;
                        let tile = targets[t];
                        let choice = match action {
                            CreatureGroundAction::MoveCreature => Action::MoveCreature {
                                creature: agent.creature,
                                destination: position_of(tile),
                            }
                            .into(),
                            CreatureGroundAction::Delay { .. } => {
                                Tuning::Delay { behaviour: b }.into()
                            }
                        };
                        (scores, b, d, choice)
                    }
                },
            );

            agent.thinking = thinking;
            agent.history.insert(decision, Instant::now());

            match choice {
                Choice::Nothing => {}
                Choice::Action(action) => {
                    actions.push(action);
                }
                Choice::Tuning(tuning) => match tuning {
                    Tuning::Delay { .. } => {}
                },
            }
        }
        // for agent in self.invaser_agents.iter_mut() {
        //     for behaviour in &behaviours.invaser_animal {
        //         let (animal, action) = agent.consider(&behaviour.decisions, &self.creatures);
        //         match action {
        //             InvaserCreatureAction::Bite => {}
        //         }
        //     }
        // }
        actions
    }

    pub fn get_creature_agents(&self) -> Vec<AgentRef> {
        self.creature_agents
            .iter()
            .map(|agent| AgentRef {
                id: agent.creature.id,
            })
            .collect()
    }

    pub fn get_creature_agent(&self, id: usize) -> Option<&CreatureAgent> {
        self.creature_agents
            .iter()
            .find(|agent| agent.creature.id == id)
    }
}

#[inline]
pub fn position_of(tile: [usize; 2]) -> [f32; 2] {
    [tile[0] as f32 + 0.5, tile[1] as f32 + 0.5]
}
