use crate::{CreatureCropInput, CreatureGroundInput, CropView};
use game::math::{TileMath, VectorMath};
use game::model::Creature;
use game::physics::SpaceId;
use rand::{thread_rng, Rng};

pub struct Fauna {
    creature: Creature,
    space: SpaceId,
    hunger: f32,
    position: [f32; 2],
    radius: usize,
}

type Crop = CreatureCropInput;

type Ground = CreatureGroundInput;

impl Fauna {
    pub fn crop(&self, input: Crop, crop: &CropView) -> f32 {
        match input {
            Crop::Hunger => self.hunger,
            Crop::CropDistance => crop.position.distance(self.position) / 10.0,
            Crop::CropNutritionValue => crop.growth / 5.0,
            Crop::Constant => 0.0,
        }
    }

    pub fn ground(&self, input: Ground, tile: [usize; 2]) -> f32 {
        match input {
            Ground::Constant => 1.0,
            Ground::Random => thread_rng().gen_range(0.0..=1.0),
            Ground::Cooldown(start, end) => 1.0,
            Ground::Distance => self.position.distance(tile.position()) / 10.0,
        }
    }
}
