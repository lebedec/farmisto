use std::collections::{HashMap, HashSet};

use game::api::Action;
use game::math::{ArrayIndex, Random, TileMath, VectorMath};
use game::model::{Creature, Farmland};
use game::raising::{Behaviour, TetherId};

use crate::decision_making::{Decision, Thinking};
use crate::perception::{CreatureView, FoodView, Owner};
use crate::{decision_making, CropView, Nature};

pub struct CreatureAgent {
    pub entity: Creature,
    pub farmland: Farmland,
    pub thinking: Thinking,
    pub targeting: Targeting,
    pub position: [f32; 2],
    pub interaction: f32,
    pub radius: f32,
    pub hunger: f32,
    pub health: f32,
    pub thirst: f32,
    pub colonization_date: f32,
    pub daytime: f32,
    pub behaviour: Behaviour,
    pub timestamps: HashMap<Behaviour, f32>,
    pub cooldowns: HashMap<String, f32>,
    pub disabling: HashSet<String>,
    pub tether: Option<TetherId>,
}

#[derive(Default, Clone, serde::Serialize)]
pub struct Targeting {
    pub crops: Vec<usize>,
    pub tiles: Vec<[usize; 2]>,
    pub foods: Vec<usize>,
    pub me: Vec<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MyAction {
    Nothing,
    Relax,
    Enable(String),
    Disable(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum My {
    Constant,
    Delay(Behaviour, f32),
    Hunger,
    Thirst,
    Daytime,
    Disabling(String),
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CropAction {
    Eat,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum Crop {
    Constant,
    Hunger,
    Distance,
    Nutrition,
    Delay(Behaviour, f32),
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum FoodAction {
    Eat,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum Food {
    Hunger,
    Distance,
    Nutrition,
    Delay(Behaviour, f32),
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum GroundAction {
    Move,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Ground {
    Constant,
    Random,
    Delay(Behaviour, f32),
    Cooldown(String, f32),
    Distance,
    Daytime,
    Feeding,
    Tethering,
    HasTethering,
}

pub enum Tuning {
    Nothing,
    Enable(String),
    Disable(String),
}

pub type CreatureDecision = Decision<My, MyAction>;
pub type CreatureCropDecision = Decision<Crop, CropAction>;
pub type CreatureFoodDecision = Decision<Food, FoodAction>;
pub type CreatureGroundDecision = Decision<Ground, GroundAction>;

type Reaction = decision_making::Reaction<Action, Tuning>;

impl CreatureAgent {
    pub fn tune(&mut self, tuning: Tuning) {
        match tuning {
            Tuning::Nothing => {}
            Tuning::Enable(behaviour) => {
                self.disabling.remove(&behaviour);
            }
            Tuning::Disable(behaviour) => {
                self.disabling.insert(behaviour);
            }
        }
    }

    pub fn me(&self, input: &My, me: &CreatureView, context: &Nature) -> f32 {
        match input {
            My::Constant => 1.0,
            My::Hunger => self.hunger,
            My::Thirst => self.thirst,
            My::Delay(behaviour, delay) => self.duration(*behaviour) / delay,
            My::Daytime => self.daytime,
            My::Disabling(behaviour) => self.disabled(behaviour),
        }
    }

    pub fn react_me(&self, action: &MyAction, me: &CreatureView) -> Reaction {
        let action = match action {
            MyAction::Nothing => return Reaction::Tuning(Tuning::Nothing),
            MyAction::Relax => Action::TakeNap {
                creature: self.entity,
            },
            MyAction::Enable(behaviour) => {
                return Reaction::Tuning(Tuning::Enable(behaviour.clone()))
            }
            MyAction::Disable(behaviour) => {
                return Reaction::Tuning(Tuning::Disable(behaviour.clone()))
            }
        };
        Reaction::Action(action)
    }

    pub fn crop(&self, input: &Crop, crop: &CropView, context: &Nature) -> f32 {
        match input {
            Crop::Hunger => self.hunger,
            Crop::Distance => crop.position.distance(self.position) / self.interaction,
            Crop::Nutrition => crop.growth / 5.0,
            Crop::Constant => 1.0,
            Crop::Delay(behaviour, delay) => self.duration(*behaviour) / delay,
        }
    }

    pub fn react_crop(&self, action: &CropAction, crop: &CropView) -> Reaction {
        let action = match action {
            CropAction::Eat => Action::EatCrop {
                crop: crop.entity,
                creature: self.entity,
            },
        };
        Reaction::Action(action)
    }

    pub fn food(&self, input: &Food, food: &FoodView, context: &Nature) -> f32 {
        match input {
            Food::Hunger => self.hunger,
            Food::Distance => self.position.distance(food.position) / self.interaction,
            Food::Nutrition => food.quantity as f32 / food.max_quantity as f32,
            Food::Delay(behaviour, delay) => self.duration(*behaviour) / delay,
        }
    }

    pub fn react_food(&self, action: &FoodAction, food: &FoodView) -> Reaction {
        let action = match action {
            FoodAction::Eat => match food.owner {
                Owner::Stack(stack) => Action::EatFoodFromStack {
                    creature: self.entity,
                    stack,
                    item: food.item,
                },
                Owner::Hands(farmer) => Action::EatFoodFromHands {
                    creature: self.entity,
                    farmer,
                    item: food.item,
                },
            },
        };
        Reaction::Action(action)
    }

    pub fn ground(&self, input: &Ground, tile: &[usize; 2], context: &Nature) -> f32 {
        match input {
            Ground::Constant => 1.0,
            Ground::Random => Random::new().generate(),
            Ground::Delay(behaviour, delay) => self.duration(*behaviour) / delay,
            Ground::Distance => self.position.distance(tile.position()) / self.radius,
            Ground::Daytime => self.daytime,
            Ground::Cooldown(tag, cooldown) => self.cooldown(tag) / cooldown,
            Ground::Feeding => context.feeding_map[tile.fit(128)],
            Ground::Tethering => context.get_tethering(tile.position(), &self.tether),
            Ground::HasTethering => {
                if self.tether.is_some() {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn react_ground(&self, action: &GroundAction, tile: &[usize; 2]) -> Reaction {
        match action {
            GroundAction::Move => Reaction::Action(Action::MoveCreature {
                creature: self.entity,
                destination: tile.position(),
            }),
        }
    }
}

impl CreatureAgent {
    fn disabled(&self, behavior: &str) -> f32 {
        if self.disabling.contains(behavior) {
            1.0
        } else {
            0.0
        }
    }

    fn duration(&self, behaviour: Behaviour) -> f32 {
        let timestamp = *self.timestamps.get(&behaviour).unwrap_or(&0.0);
        self.colonization_date - timestamp
    }

    fn cooldown(&self, tag: &str) -> f32 {
        let timestamp = *self.cooldowns.get(tag).unwrap_or(&0.0);
        self.colonization_date - timestamp
    }
}
