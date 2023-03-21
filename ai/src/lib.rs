use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{format, Debug};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info};
use rand::{thread_rng, Rng};
use serde::Serialize;

use crate::api::serve_web_socket;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::VectorMath;
use game::model::{Creature, Crop, Universe};
use game::physics::{Physics, SpaceId};
use network::TcpClient;

mod api;

pub struct AiThread {}

impl AiThread {
    pub fn spawn(mut client: TcpClient, behaviours: Arc<RwLock<Behaviours>>) -> Self {
        let nature = Nature {
            crops: vec![],
            creatures: vec![],
            creature_agents: vec![],
            invaser_agents: vec![],
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
                        match action {
                            Action::Nothing => {
                                continue;
                            }
                            _ => {}
                        }
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Curve {
    x: Vec<f32>,
    y: Vec<f32>,
}

impl Curve {
    pub fn respond(&self, mut t: f32) -> f32 {
        if t < 0.0 {
            t = 0.0;
        }
        if t > 1.0 {
            t = 1.0;
        }
        for (index, x) in self.x.iter().enumerate() {
            let x = *x;
            if x > t || x >= 1.0 {
                let start = index - 1;
                let end = index;
                let segment = self.x[end] - self.x[start];
                let progress = (t - self.x[start]) / segment;
                let delta = self.y[end] - self.y[start];
                let value = self.y[start] + delta * progress;
                return value;
            }
        }
        1.0
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Consideration<I>
where
    I: Sized + Serialize,
{
    pub input: I,
    pub curve: Curve,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Decision<I, A>
where
    A: Copy + Sized + Debug + Serialize,
    I: Sized + Serialize,
{
    pub action: A,
    pub considerations: Vec<Consideration<I>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Behaviour<D>
where
    D: Sized,
{
    name: String,
    decisions: Vec<D>,
}

pub struct CropView {
    entity: Crop,
    growth: f32,
    position: [f32; 2],
}

pub struct FarmerView {}

pub struct CreatureView {
    entity: Creature,
}

pub struct InvaserView {
    threat: f32,
}

pub struct CreatureAgent {
    creature: Creature,
    space: SpaceId,
    hunger: f32,
    mindset: Vec<String>,
    thinking: Thinking,
    position: [f32; 2],
    radius: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentRef {
    id: usize,
}

/// Represents a decision making report via last update.
#[derive(Default, Clone, serde::Serialize)]
pub struct Thinking {
    pub reasons: HashMap<String, f32>,
    pub current_set: usize,
}

impl Thinking {
    pub fn reason(&mut self, key: String, score: f32) {
        let set = self.current_set;
        self.reasons.insert(format!("{set}:{key}"), score);
    }
}

fn make_decision<S, C>(behaviour_sets: &Vec<S>, consider: C) -> (Option<Action>, Thinking)
where
    C: Fn(&S, &mut Thinking) -> (f32, Action),
{
    let mut thinking = Thinking::default();
    let mut best_action = None;
    let mut best_action_scores = 0.0;
    for (set_index, set) in behaviour_sets.iter().enumerate() {
        thinking.current_set = set_index;
        let (scores, action) = consider(set, &mut thinking);
        if scores > best_action_scores {
            best_action = Some(action);
            best_action_scores = scores;
        }
    }
    (best_action, thinking)
}

fn consider<I, T, F, A>(
    behaviours: &Vec<Behaviour<Decision<I, A>>>,
    targets: &Vec<T>,
    input: F,
    thinking: &mut Thinking,
) -> (usize, usize, usize, f32)
where
    A: Copy + Sized + Debug + Serialize,
    I: Copy + Sized + Serialize,
    F: Fn(I, &T) -> f32,
{
    let mut best_behaviour = 0;
    let mut best_behaviour_decision = 0;
    let mut best_behaviour_target = 0;
    let mut best_behaviour_scores = 0.0;
    for (behaviour_index, behaviour) in behaviours.iter().enumerate() {
        let mut best_target = 0;
        let mut best_target_decision = 0;
        let mut best_target_scores = 0.0;
        for (target_index, target) in targets.iter().enumerate() {
            let mut best_decision = 0;
            let mut best_decision_scores = 0.0;
            for (decision_index, decision) in behaviour.decisions.iter().enumerate() {
                let mut scores = 1.0;
                for (index, consideration) in decision.considerations.iter().enumerate() {
                    let x = input(consideration.input, target);
                    let score = consideration.curve.respond(x);
                    {
                        // TODO: exclude from release build
                        let key =
                            format!("{behaviour_index}:{target_index}:{decision_index}:{index}");
                        thinking.reason(key, x);
                    }
                    scores *= score;
                    if scores == 0.0 {
                        // optimization:
                        // skip considerations for obviously zero scored decision
                        break;
                    }
                }
                if scores > best_decision_scores {
                    best_decision_scores = scores;
                    best_decision = decision_index;
                }
                if best_decision_scores > 0.95 {
                    // optimization:
                    // no need to compare decisions very precisely if we found one good enough
                    break;
                }
            }
            if best_decision_scores > best_target_scores {
                best_target = target_index;
                best_target_decision = best_decision;
                best_target_scores = best_decision_scores;
            }
            if best_target_scores > 0.95 {
                // optimization:
                // no need to choose a target very precisely if we found one good enough
                break;
            }
        }
        if best_target_scores > best_behaviour_scores {
            best_behaviour = behaviour_index;
            best_behaviour_decision = best_target_decision;
            best_behaviour_target = best_target;
            best_behaviour_scores = best_target_scores;
        }
        if best_behaviour_scores > 0.95 {
            // optimization:
            // not need to consider every behaviour if we found one appropriate enough
            break;
        }
    }
    (
        best_behaviour,
        best_behaviour_target,
        best_behaviour_decision,
        best_behaviour_scores,
    )
}

pub struct InvaserAgent {
    mindset: Vec<String>,
}

impl InvaserAgent {
    pub fn consider(
        &self,
        decisions: &Vec<InvaserAnimalDecision>,
        creatures: &Vec<CreatureView>,
    ) -> (usize, InvaserAnimalAction) {
        (0, InvaserAnimalAction::Bite)
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimalCropAction {
    Nothing,
    EatCrop,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimalCropInput {
    Constant,
    Hunger,
    CropDistance,
    CropNutritionValue,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimalGroundAction {
    MoveCreature,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimalGroundInput {
    Constant,
    Random { min: f32, max: f32 },
    Cooldown(usize),
    Distance,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum InvaserAnimalAction {
    Bite,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum InvaserAnimalInput {
    AnimalDistance,
}

type AnimalCropDecision = Decision<AnimalCropInput, AnimalCropAction>;
type AnimalGroundDecision = Decision<AnimalGroundInput, AnimalGroundAction>;
type InvaserAnimalDecision = Decision<InvaserAnimalInput, InvaserAnimalAction>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Behaviours {
    invaser_animal: Vec<Behaviour<InvaserAnimalDecision>>,
    animals: Vec<AnimalBehaviourSet>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum AnimalBehaviourSet {
    Crop {
        name: String,
        behaviours: Vec<Behaviour<AnimalCropDecision>>,
    },
    Ground {
        name: String,
        behaviours: Vec<Behaviour<AnimalGroundDecision>>,
    },
}

#[derive(Default, Clone, Copy)]
struct Tile {
    has_hole: bool,
    has_barrier: bool,
}

pub struct Nature {
    // game view
    crops: Vec<CropView>,
    creatures: Vec<CreatureView>,
    // agents
    creature_agents: Vec<CreatureAgent>,
    invaser_agents: Vec<InvaserAgent>,
    tiles: HashMap<SpaceId, Vec<Vec<Tile>>>,
}

impl Nature {
    pub fn perceive(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::Universe(events) => {
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
                            Universe::DropAppeared { .. } => {}
                            Universe::DropVanished(_) => {}
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
                                health,
                                position,
                                hunger,
                                space,
                                ..
                            } => {
                                self.creatures.push(CreatureView { entity });
                                self.creature_agents.push(CreatureAgent {
                                    creature: entity,
                                    space,
                                    hunger,
                                    mindset: vec![],
                                    thinking: Thinking::default(),
                                    position,
                                    radius: 5,
                                })
                            }
                            Universe::CreatureEats { entity } => {}
                            Universe::CreatureVanished(_) => {}
                        }
                    }
                }
                Event::Physics(events) => {
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
                                space,
                                id,
                                position,
                                bounds,
                            } => {
                                let tiles = self.tiles.get_mut(&space).unwrap();
                                let [x, y] = position.to_tile();
                                // TODO: barrier bounds
                                tiles[y][x].has_barrier = true;
                            }
                            Physics::BarrierDestroyed {
                                id,
                                position,
                                space,
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
                Event::Building(_) => {}
                Event::Inventory(_) => {}
                Event::Planting(_) => {}
                Event::Raising(_) => {}
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
            let (action, thinking) =
                make_decision(&behaviours.animals, |set, thinking| match set {
                    AnimalBehaviourSet::Crop { behaviours, .. } => {
                        let (b, t, d, scores) = consider(
                            &behaviours,
                            &self.crops,
                            |input, crop| match input {
                                AnimalCropInput::Hunger => agent.hunger,
                                AnimalCropInput::CropDistance => {
                                    crop.position.distance(agent.position) / 10.0
                                }
                                AnimalCropInput::CropNutritionValue => crop.growth / 5.0,
                                AnimalCropInput::Constant => 0.0,
                            },
                            thinking,
                        );
                        let action = behaviours[b].decisions[d].action;
                        let crop = self.crops[t].entity;
                        let action = match action {
                            AnimalCropAction::EatCrop => Action::EatCrop {
                                crop,
                                creature: agent.creature,
                            },
                            AnimalCropAction::Nothing => Action::Nothing,
                        };
                        (scores, action)
                    }
                    AnimalBehaviourSet::Ground { behaviours, .. } => {
                        let game_tiles = self.tiles.get(&agent.space).unwrap();
                        let targets = Self::get_empty_tiles(
                            game_tiles,
                            agent.position.to_tile(),
                            agent.radius,
                        );
                        let (b, t, d, scores) = consider(
                            &behaviours,
                            &targets,
                            |input, tile| match input {
                                AnimalGroundInput::Constant => 0.0,
                                AnimalGroundInput::Random { min, max } => {
                                    thread_rng().gen_range(min..max)
                                }
                                AnimalGroundInput::Cooldown(_) => 1.0,
                                AnimalGroundInput::Distance => {
                                    agent.position.distance(position_of(*tile)) / 10.0
                                }
                            },
                            thinking,
                        );
                        let action = behaviours[b].decisions[d].action;
                        let tile = targets[t];
                        let action = match action {
                            AnimalGroundAction::MoveCreature => Action::MoveCreature {
                                creature: agent.creature,
                                destination: position_of(tile),
                            },
                        };
                        (scores, action)
                    }
                });
            agent.thinking = thinking;

            if let Some(action) = action {
                actions.push(action);
            }
        }
        for agent in self.invaser_agents.iter_mut() {
            for behaviour in &behaviours.invaser_animal {
                let (animal, action) = agent.consider(&behaviour.decisions, &self.creatures);
                match action {
                    InvaserAnimalAction::Bite => {}
                }
            }
        }
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
