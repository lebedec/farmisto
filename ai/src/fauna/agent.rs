use crate::decision_making::{Decision, Thinking};
use crate::{decision_making, CropView};
use game::api::Action;
use game::math::{TileMath, VectorMath};
use game::model::Creature;
use game::physics::SpaceId;
use rand::{thread_rng, Rng};

pub struct CreatureAgent {
    pub creature: Creature,
    pub space: SpaceId,
    pub hunger: f32,
    pub thinking: Thinking,
    pub position: [f32; 2],
    pub radius: usize,
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
    Delay,
}

pub type CreatureCropDecision = Decision<Crop, CropAction>;
pub type CreatureGroundDecision = Decision<Ground, GroundAction>;

type Reaction = decision_making::Reaction<Action, Tuning>;

impl CreatureAgent {
    pub fn tune(&mut self, tuning: Tuning) {
        match tuning {
            Tuning::Delay => {}
        }
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
                creature: self.creature,
            },
            CropAction::Nothing => Action::EatCrop {
                crop: crop.entity,
                creature: self.creature,
            },
        };
        Reaction::Action(action)
    }

    pub fn ground(&self, input: Ground, tile: &[usize; 2]) -> f32 {
        match input {
            Ground::Constant => 1.0,
            Ground::Random => thread_rng().gen_range(0.0..=1.0),
            Ground::Cooldown(start, end) => 1.0,
            Ground::Distance => self.position.distance(tile.position()) / self.radius as f32,
        }
    }

    pub fn react_ground(&self, action: GroundAction, tile: &[usize; 2]) -> Reaction {
        match action {
            GroundAction::Move => Reaction::Action(Action::MoveCreature {
                creature: self.creature,
                destination: tile.position(),
            }),
            GroundAction::Delay { .. } => Reaction::Tuning(Tuning::Delay),
        }
    }
}
