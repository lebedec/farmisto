use crate::decision_making::{Decision, Thinking};
use crate::perception::{CreatureView, FoodContainer, FoodView};
use crate::{decision_making, CropView};
use game::api::Action;
use game::math::{TileMath, VectorMath};
use game::model::{Creature, Farmland};
use game::raising::Behaviour;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::time::Instant;

pub struct CreatureAgent {
    pub entity: Creature,
    pub farmland: Farmland,
    pub thinking: Thinking,
    pub position: [f32; 2],
    pub radius: f32,
    pub hunger: f32,
    pub health: f32,
    pub thirst: f32,
    pub colonization_date: f32,
    pub daytime: f32,
    pub behaviour: Behaviour,
    pub timestamps: HashMap<Behaviour, f32>,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum MyAction {
    Nothing,
    Relax,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum My {
    Constant,
    Delay(Behaviour, f32),
    Hunger,
    Thirst,
    Daytime,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CropAction {
    Nothing,
    Eat,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum Crop {
    Constant,
    Hunger,
    Distance,
    Nutrition,
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
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum GroundAction {
    Move,
    Delay { min: f32, max: f32 },
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum Ground {
    Constant,
    Random,
    Cooldown(f32, f32),
    Distance,
}

pub enum Tuning {
    Nothing,
    Delay,
}

pub type CreatureDecision = Decision<My, MyAction>;
pub type CreatureCropDecision = Decision<Crop, CropAction>;
pub type CreatureFoodDecision = Decision<Food, FoodAction>;
pub type CreatureGroundDecision = Decision<Ground, GroundAction>;

type Reaction = decision_making::Reaction<Action, Tuning>;

impl CreatureAgent {
    pub fn tune(&mut self, tuning: Tuning) {
        match tuning {
            Tuning::Delay => {}
            Tuning::Nothing => {}
        }
    }

    pub fn me(&self, input: My, me: &CreatureView) -> f32 {
        match input {
            My::Constant => 1.0,
            My::Hunger => self.hunger,
            My::Thirst => self.thirst,
            My::Delay(behaviour, delay) => self.duration(behaviour) / delay,
            My::Daytime => self.daytime,
        }
    }

    pub fn react_me(&self, action: MyAction, me: &CreatureView) -> Reaction {
        let action = match action {
            MyAction::Nothing => return Reaction::Tuning(Tuning::Nothing),
            MyAction::Relax => Action::TakeNap {
                creature: self.entity,
            },
        };
        Reaction::Action(action)
    }

    pub fn crop(&self, input: Crop, crop: &CropView) -> f32 {
        match input {
            Crop::Hunger => self.hunger,
            Crop::Distance => crop.position.distance(self.position) / 10.0,
            Crop::Nutrition => crop.growth / 5.0,
            Crop::Constant => 1.0,
        }
    }

    pub fn react_crop(&self, action: CropAction, crop: &CropView) -> Reaction {
        let action = match action {
            CropAction::Eat => Action::EatCrop {
                crop: crop.entity,
                creature: self.entity,
            },
            CropAction::Nothing => Action::EatCrop {
                crop: crop.entity,
                creature: self.entity,
            },
        };
        Reaction::Action(action)
    }

    pub fn food(&self, input: Food, food: &FoodView) -> f32 {
        match input {
            Food::Hunger => self.hunger,
            Food::Distance => self.position.distance(food.position) / self.radius,
            Food::Nutrition => food.quantity as f32 / food.max_quantity as f32,
        }
    }

    pub fn react_food(&self, action: FoodAction, food: &FoodView) -> Reaction {
        let action = match action {
            FoodAction::Eat => match food.container_entity {
                FoodContainer::Stack(stack) => Action::EatFoodFromStack {
                    creature: self.entity,
                    item: food.item,
                    stack,
                },
                FoodContainer::Hands(farmer) => Action::EatFoodFromHands {
                    creature: self.entity,
                    item: food.item,
                    farmer,
                },
            },
        };
        Reaction::Action(action)
    }

    pub fn ground(&self, input: Ground, tile: &[usize; 2]) -> f32 {
        match input {
            Ground::Constant => 1.0,
            Ground::Random => thread_rng().gen_range(0.0..=1.0),
            Ground::Cooldown(start, end) => 1.0,
            Ground::Distance => self.position.distance(tile.position()) / self.radius,
        }
    }

    pub fn react_ground(&self, action: GroundAction, tile: &[usize; 2]) -> Reaction {
        match action {
            GroundAction::Move => Reaction::Action(Action::MoveCreature {
                creature: self.entity,
                destination: tile.position(),
            }),
            GroundAction::Delay { .. } => Reaction::Tuning(Tuning::Delay),
        }
    }
}

impl CreatureAgent {
    fn duration(&self, behaviour: Behaviour) -> f32 {
        let timestamp = *self.timestamps.get(&behaviour).unwrap_or(&0.0);
        self.colonization_date - timestamp
    }
}
