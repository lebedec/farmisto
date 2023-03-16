use log::{error, info};
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use game::api::{Action, Event, GameResponse, PlayerRequest};
use game::model::{Creature, Crop, Universe};
use network::TcpClient;

pub struct AiThread {}

impl AiThread {
    pub fn spawn(mut client: TcpClient, behaviours: Arc<RwLock<Behaviours>>) -> Self {
        thread::spawn(move || {
            let mut nature = Nature {
                crops: vec![],
                creatures: vec![],
                creature_agents: vec![],
                invaser_agents: vec![],
            };
            let mut action_id = 0;
            loop {
                let t = Instant::now();
                let events = handle_server_responses(&mut client);
                nature.perceive(events);
                for action in nature.react(&behaviours.read().unwrap()) {
                    info!("AI sends id={} {:?}", action_id, action);
                    client.send(PlayerRequest::Perform { action, action_id });
                    action_id += 1;
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

#[derive(serde::Deserialize)]
pub struct Curve {
    x: Vec<f32>,
    y: Vec<f32>,
}

impl Curve {
    pub fn respond(&self, x: f32) -> f32 {
        1.0
    }
}

#[derive(serde::Deserialize)]
pub struct Consideration<I>
where
    I: Sized,
{
    pub input: I,
    pub weight: f32,
    pub curve: Curve,
}

#[derive(serde::Deserialize)]
pub struct Decision<I, A>
where
    A: Copy + Sized,
    I: Sized,
{
    pub action: A,
    pub considerations: Vec<Consideration<I>>,
}

#[derive(serde::Deserialize)]
pub struct Behaviour<D>
where
    D: Sized,
{
    decisions: Vec<D>,
}

pub struct CropView {
    entity: Crop,
    growth: f32,
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
}

impl CreatureAgent {
    pub fn consider(
        &self,
        decisions: &Vec<AnimalCropDecision>,
        crops: &Vec<CropView>,
    ) -> (Crop, AnimalCropAction) {
        let best_crop = 0;
        let best = 0;
        for crop in crops {
            for decision in decisions {
                let mut scores = 0.0;
                for consideration in &decision.considerations {
                    let x = match consideration.input {
                        AnimalCropInput::Hunger => self.hunger,
                        AnimalCropInput::CropDistance => 0.0,
                        AnimalCropInput::CropNutritionValue => crop.growth / 5.0,
                        AnimalCropInput::Constant => 1.0,
                    };
                    let score = consideration.curve.respond(x);
                    scores += score;
                }
            }
        }
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

#[derive(Copy, Clone, serde::Deserialize)]
pub enum AnimalCropAction {
    Nothing,
    EatCrop,
}

#[derive(serde::Deserialize)]
pub enum AnimalCropInput {
    Constant,
    Hunger,
    CropDistance,
    CropNutritionValue,
}

#[derive(Copy, Clone, serde::Deserialize)]
pub enum InvaserAnimalAction {
    Bite,
}

#[derive(serde::Deserialize)]
pub enum InvaserAnimalInput {
    AnimalDistance,
}

type AnimalCropDecision = Decision<AnimalCropInput, AnimalCropAction>;
type InvaserAnimalDecision = Decision<InvaserAnimalInput, InvaserAnimalAction>;

#[derive(serde::Deserialize)]
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
                            Universe::CropAppeared { entity, growth, .. } => {
                                self.crops.push(CropView { entity, growth })
                            }
                            Universe::CropVanished(_) => {}
                            Universe::ConstructionAppeared { .. } => {}
                            Universe::ConstructionVanished { .. } => {}
                            Universe::EquipmentAppeared { .. } => {}
                            Universe::EquipmentVanished(_) => {}
                            Universe::ItemsAppeared { .. } => {}
                            Universe::CreatureAppeared { entity, health, .. } => {
                                self.creatures.push(CreatureView { entity });
                                self.creature_agents.push(CreatureAgent {
                                    creature: entity,
                                    hunger: 0.0,
                                    mindset: vec![],
                                })
                            }
                            Universe::CreatureEats { entity } => {}
                            Universe::CreatureVanished(_) => {}
                        }
                    }
                }
                Event::Physics(_) => {}
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
}
