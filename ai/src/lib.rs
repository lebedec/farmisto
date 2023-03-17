use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{format, Debug};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, info};
use serde::Serialize;

use crate::api::serve_api;
use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::math::VectorMath;
use game::model::{Creature, Crop, Universe};
use game::physics::Physics;
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
        };
        let nature_lock = Arc::new(RwLock::new(nature));
        let nature_read_access = nature_lock.clone();
        thread::spawn(move || serve_api(nature_read_access));
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

                // 250 ms delay to simulate human reaction
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
    pub weight: f32,
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
    hunger: f32,
    mindset: Vec<String>,
    animal_crop: Thinking,
    position: [f32; 2],
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentRef {
    id: usize,
}

#[derive(Default, Clone, serde::Serialize)]
pub struct Thinking {
    pub scores: HashMap<String, (f32, f32)>,
    pub best_scores: f32,
    pub best_target: usize,
    pub best_decision: usize,
}

impl CreatureAgent {
    pub fn consider(
        &mut self,
        decisions: &Vec<AnimalCropDecision>,
        crops: &Vec<CropView>,
    ) -> (Crop, AnimalCropAction) {
        self.animal_crop = Thinking::default();
        let mut best_crop = 0;
        let mut best_crop_scores = 0.0;
        let mut best = 0;
        let mut best_scores = 0.0;
        for (crop_index, crop) in crops.iter().enumerate() {
            for (d_index, decision) in decisions.iter().enumerate() {
                let mut scores = 1.0;
                for (c_index, consideration) in decision.considerations.iter().enumerate() {
                    let x = match consideration.input {
                        AnimalCropInput::Hunger => self.hunger,
                        AnimalCropInput::CropDistance => {
                            crop.position.distance(self.position) / 10.0
                        }
                        AnimalCropInput::CropNutritionValue => crop.growth / 5.0,
                        AnimalCropInput::Constant => 1.0,
                    };
                    let score = consideration.curve.respond(x);
                    scores *= score;
                    let key = format!("{crop_index}:{d_index}:{c_index}");
                    self.animal_crop.scores.insert(key, (x, score));
                }
                if scores > best_scores {
                    best_scores = scores;
                    best = d_index;
                }
            }
            if best_scores > best_crop_scores {
                best_crop_scores = best_scores;
                best_crop = crop_index;
            }
        }
        self.animal_crop.best_decision = best;
        self.animal_crop.best_scores = best_crop_scores;
        self.animal_crop.best_target = best_crop;
        return (crops[best_crop].entity, decisions[best].action);
    }
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

#[derive(serde::Serialize, serde::Deserialize)]
pub enum AnimalCropInput {
    Constant,
    Hunger,
    CropDistance,
    CropNutritionValue,
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
type InvaserAnimalDecision = Decision<InvaserAnimalInput, InvaserAnimalAction>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Behaviours {
    animal_crop: Vec<Behaviour<AnimalCropDecision>>,
    invaser_animal: Vec<Behaviour<InvaserAnimalDecision>>,
}

pub struct Nature {
    // game view
    crops: Vec<CropView>,
    creatures: Vec<CreatureView>,
    // agents
    creature_agents: Vec<CreatureAgent>,
    invaser_agents: Vec<InvaserAgent>,
}

impl Nature {
    pub fn perceive(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::Universe(events) => {
                    for event in events {
                        match event {
                            Universe::ActivityChanged { .. } => {}
                            Universe::BarrierHintAppeared { .. } => {}
                            Universe::TreeAppeared { .. } => {}
                            Universe::TreeVanished(_) => {}
                            Universe::FarmlandAppeared { .. } => {}
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
                                ..
                            } => {
                                self.creatures.push(CreatureView { entity });
                                self.creature_agents.push(CreatureAgent {
                                    creature: entity,
                                    hunger: 0.0,
                                    mindset: vec![],
                                    animal_crop: Thinking::default(),
                                    position,
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
                            Physics::BarrierCreated { .. } => {}
                            Physics::BarrierDestroyed { .. } => {}
                            Physics::SpaceUpdated { .. } => {}
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

    pub fn react(&mut self, behaviours: &Behaviours) -> Vec<Action> {
        let mut actions = vec![];
        for agent in self.creature_agents.iter_mut() {
            for behaviour in &behaviours.animal_crop {
                let (crop, action) = agent.consider(&behaviour.decisions, &self.crops);
                match action {
                    AnimalCropAction::EatCrop => actions.push(Action::EatCrop {
                        crop,
                        creature: agent.creature,
                    }),
                    AnimalCropAction::Nothing => {}
                }
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

    pub fn get_creature_agent(&mut self, id: usize) -> Option<&CreatureAgent> {
        self.creature_agents
            .iter()
            .find(|agent| agent.creature.id == id)
    }
}
