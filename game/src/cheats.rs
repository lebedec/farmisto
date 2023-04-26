use crate::api::{ActionError, Event};
use crate::math::VectorMath;
use crate::model::{Farmer, Farmland};
use crate::Game;

impl Game {
    pub(crate) fn cheat_growth_up_crops(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        growth: f32,
        radius: f32,
    ) -> Result<Vec<Event>, ActionError> {
        let center = self.physics.get_body(farmer.body)?.position;
        for crop in self.universe.crops.iter() {
            let position = self.physics.get_barrier(crop.barrier)?.position;
            if center.distance(position) < radius {
                self.planting.get_plant_mut(crop.plant).unwrap().growth = growth;
            }
        }
        Ok(vec![])
    }
}
