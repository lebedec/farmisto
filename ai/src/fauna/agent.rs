use crate::decision_making::{Decision, Thinking};
use crate::perception::{CreatureView, FoodView};
use crate::{decision_making, CropView};
use game::api::Action;
use game::math::{TileMath, VectorMath};
use game::model::Creature;
use game::physics::SpaceId;
use log::info;
use rand::{thread_rng, Rng};
use std::time::Instant;

pub struct CreatureAgent {
    pub entity: Creature,
    pub last_action: Instant,
    pub space: SpaceId,
    pub thinking: Thinking,
    pub position: [f32; 2],
    pub radius: f32,
    pub hunger: f32,
    pub health: f32,
    pub thirst: f32,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum MyAction {
    Nothing,
    Relax,
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum My {
    Idle,
    Hunger,
    Thirst,
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
    Idle,
}

pub enum Tuning {
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
        }
    }

    pub fn me(&self, input: My, me: &CreatureView) -> f32 {
        match input {
            My::Hunger => self.hunger,
            My::Thirst => self.thirst,
            My::Idle => 1.0,
        }
    }

    pub fn react_me(&self, action: MyAction, me: &CreatureView) -> Reaction {
        let action = match action {
            MyAction::Nothing => Action::TakeNap {
                creature: self.entity,
            },
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
            FoodAction::Eat => Action::EatFood {
                creature: self.entity,
                item: food.item,
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
            Ground::Idle => self.last_action.elapsed().as_secs_f32() / 3.0,
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
