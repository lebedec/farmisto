use crate::planting::Planting::{PlantDamaged, PlantFruitsChanged, PlantUpdated};
use crate::planting::{PlantId, Planting, PlantingDomain, PlantingError, SoilId};

impl PlantingDomain {
    pub fn update_impact(&mut self) {}

    pub fn request_fertility_consumption(
        &mut self,
        soil: SoilId,
        place: usize,
        expected: f32,
    ) -> Result<f32, PlantingError> {
        let soil = self.get_soil_mut(soil)?;
        let fertility = soil.fertility[place];
        let delta = expected.min(fertility);
        soil.fertility[place] -= delta;
        Ok(delta)
    }

    pub fn integrate_thirst(
        &mut self,
        id: PlantId,
        lack: f32,
        consumption: f32,
    ) -> Result<(), PlantingError> {
        let plant = self.get_plant_mut(id)?;
        if lack > 0.0 {
            plant.thirst = (plant.thirst + lack).min(1.0);
        } else {
            plant.thirst = (plant.thirst - consumption).max(0.0);
        }
        Ok(())
    }

    pub fn integrate_hunger(
        &mut self,
        id: PlantId,
        lack: f32,
        consumption: f32,
    ) -> Result<(), PlantingError> {
        let plant = self.get_plant_mut(id)?;
        if lack > 0.0 {
            plant.hunger = (plant.hunger + lack).min(1.0);
        } else {
            plant.hunger = (plant.hunger - consumption).max(0.0);
        }
        Ok(())
    }

    pub fn integrate_impact(&mut self, id: PlantId, impact: f32) -> Result<(), PlantingError> {
        let plant = self.get_plant_mut(id)?;
        plant.impact += impact;
        if plant.impact < -1.0 {
            plant.impact = -1.0;
        }
        if plant.impact > 1.0 {
            plant.impact = 1.0;
        }
        Ok(())
    }

    pub fn update(&mut self, time: f32) -> Vec<Planting> {
        let mut events = vec![];

        for plants in &mut self.plants {
            for plant in plants.iter_mut() {
                let mut plant_updated = false;
                if plant.impact.abs() > 0.001 {
                    let delta = time * plant.kind.flexibility;
                    plant.impact = if delta > plant.impact.abs() {
                        0.0
                    } else {
                        plant.impact - (plant.impact.signum() * time * plant.kind.flexibility)
                    };
                    plant.health = (plant.health - time * 0.06).clamp(0.1, 1.0);
                    plant_updated = true;

                    events.push(PlantDamaged {
                        id: plant.id,
                        health: plant.health,
                    });
                }

                plant_updated = true;

                let growth = time
                    * (1.0 / (plant.kind.growth * 60.0))
                    * (1.0 - plant.thirst)
                    * (1.0 - plant.hunger);

                plant.growth = (plant.growth + growth).min(5.0);

                if plant.growth >= 2.0 && plant.growth < 3.0 {
                    plant.fruits -= time * 2.0 * plant.hunger * (1.0 / (plant.kind.growth * 60.0));
                    plant.fruits = plant.fruits.clamp(0.0, plant.kind.max_fruits);
                    events.push(PlantFruitsChanged {
                        id: plant.id,
                        fruits: plant.fruits,
                    })
                }

                if plant_updated {
                    events.push(PlantUpdated {
                        id: plant.id,
                        impact: plant.impact,
                        thirst: plant.thirst,
                        hunger: plant.hunger,
                        growth: plant.growth,
                    })
                }
            }
        }
        events
    }
}
