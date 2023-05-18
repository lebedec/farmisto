use crate::api::{ActionError, Event};
use crate::math::{TileMath, VectorMath};
use crate::model::{Farmer, Farmland};
use crate::physics::BodyId;
use crate::{emit, Game};

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

    pub(crate) fn cheat_spawn_lama(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let creature_kind = self.known.creatures.find("lama")?;
        let body = self.physics.bodies_sequence.introduce().one(BodyId);
        let create_body = self.physics.create_body(
            body,
            farmland.space,
            creature_kind.body.clone(),
            tile.position(),
        )?;
        let (animal, create_animal) = self.raising.create_animal(creature_kind.animal.clone())?;

        emit![
            create_body(),
            create_animal(),
            self.appear_creature(creature_kind.id, body, animal)?
        ]
    }

    pub(crate) fn cheat_set_creatures_health(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        health: f32,
        radius: f32,
    ) -> Result<Vec<Event>, ActionError> {
        let center = self.physics.get_body(farmer.body)?.position;
        for creature in self.universe.creatures.iter() {
            let position = self.physics.get_body(creature.body)?.position;
            if center.distance(position) < radius {
                self.raising.get_animal_mut(creature.animal).unwrap().health = health;
            }
        }
        Ok(vec![])
    }
}
