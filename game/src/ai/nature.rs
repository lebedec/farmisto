use crate::api::{Action, Event};
use crate::model::{Crop, Universe};
use serde::Serialize;
use std::fs;

#[derive(serde::Serialize)]
pub struct Curve {
    x: Vec<f32>,
    y: Vec<f32>,
}

impl Curve {
    pub fn respond(&self, x: f32) -> f32 {
        1.0
    }
}

#[derive(serde::Serialize)]
pub struct Consideration<I>
where
    I: Serialize,
{
    pub input: I,
    pub weight: f32,
    pub curve: Curve,
}

#[derive(serde::Serialize)]
pub struct Decision<I, A>
where
    A: Copy + Serialize,
    I: Serialize,
{
    pub action: A,
    pub considerations: Vec<Consideration<I>>,
}

#[derive(serde::Serialize)]
pub struct Behaviour<D>
where
    D: Serialize,
{
    decisions: Vec<D>,
}

pub struct CropView {
    entity: Crop,
    growth: f32,
}

pub struct FarmerView {}

pub struct AnimalView {
    authority: f32,
}

pub struct InvaserView {
    threat: f32,
}

pub struct AnimalAgent {
    animal: usize,
    hunger: f32,
    mindset: Vec<String>,
}

impl AnimalAgent {
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
                        AnimalCropInput::MyHunger => self.hunger,
                        AnimalCropInput::CropDistance => 0.0,
                        AnimalCropInput::CropNutritionValue => crop.growth / 5.0,
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
        animals: &Vec<AnimalView>,
    ) -> (usize, InvaserAnimalAction) {
        (0, InvaserAnimalAction::Bite)
    }
}

#[derive(Copy, Clone, Serialize)]
pub enum AnimalCropAction {
    EatCrop,
}

#[derive(serde::Serialize)]
pub enum AnimalCropInput {
    MyHunger,
    CropDistance,
    CropNutritionValue,
}

#[derive(Copy, Clone, Serialize)]
pub enum InvaserAnimalAction {
    Bite,
}

#[derive(serde::Serialize)]
pub enum InvaserAnimalInput {
    AnimalDistance,
}

type AnimalCropDecision = Decision<AnimalCropInput, AnimalCropAction>;
type InvaserAnimalDecision = Decision<InvaserAnimalInput, InvaserAnimalAction>;

#[derive(serde::Serialize)]
pub struct Behaviours {
    animal_crop: Vec<Behaviour<AnimalCropDecision>>,
    invaser_animal: Vec<Behaviour<InvaserAnimalDecision>>,
}

pub struct Nature {
    // game view
    crops: Vec<CropView>,
    animals: Vec<AnimalView>,
    // agents
    animal_agents: Vec<AnimalAgent>,
    invaser_agents: Vec<InvaserAgent>,
    // definition
    behaviours: Behaviours,
}

impl Nature {
    pub fn test() {
        let behaviours = Behaviours {
            animal_crop: vec![
                Behaviour {
                    decisions: vec![Decision {
                        action: AnimalCropAction::EatCrop,
                        considerations: vec![
                            Consideration {
                                input: AnimalCropInput::MyHunger,
                                weight: 0.0,
                                curve: Curve {
                                    x: vec![0.0, 1.0, 2.0],
                                    y: vec![5.0; 50],
                                },
                            },
                            Consideration {
                                input: AnimalCropInput::CropNutritionValue,
                                weight: 1.0,
                                curve: Curve {
                                    x: vec![0.0, 1.0, 2.0],
                                    y: vec![5.0; 50],
                                },
                            },
                        ],
                    }],
                },
                Behaviour { decisions: vec![] },
            ],
            invaser_animal: vec![Behaviour {
                decisions: vec![Decision {
                    action: InvaserAnimalAction::Bite,
                    considerations: vec![Consideration {
                        input: InvaserAnimalInput::AnimalDistance,
                        weight: 0.0,
                        curve: Curve {
                            x: vec![0.0, 1.0, 2.0],
                            y: vec![5.0; 50],
                        },
                    }],
                }],
            }],
        };
        fs::write(
            "ai.json",
            serde_json::to_string_pretty(&behaviours).unwrap(),
        );
    }

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
                        }
                    }
                }
                Event::Physics(_) => {}
                Event::Building(_) => {}
                Event::Inventory(_) => {}
                Event::Planting(_) => {}
            }
        }
    }

    pub fn update(&mut self) -> Vec<Action> {
        let mut actions = vec![];
        for agent in self.animal_agents.iter_mut() {
            for behaviour in &self.behaviours.animal_crop {
                let (crop, action) = agent.consider(&behaviour.decisions, &self.crops);
                match action {
                    AnimalCropAction::EatCrop => actions.push(Action::EatCrop {
                        crop,
                        animal: agent.animal,
                    }),
                }
            }
        }
        for agent in self.invaser_agents.iter_mut() {
            for behaviour in &self.behaviours.invaser_animal {
                let (animal, action) = agent.consider(&behaviour.decisions, &self.animals);
                match action {
                    InvaserAnimalAction::Bite => {}
                }
            }
        }
        actions
    }
}
