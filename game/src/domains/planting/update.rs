use crate::planting::Planting::{PlantDamaged, PlantUpdated};
use crate::planting::{PlantId, Planting, PlantingDomain, PlantingError};
use log::info;

impl PlantingDomain {
    pub fn update_impact(&mut self) {}

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

                // if plant.thirst <= 1.0 {
                //     plant.thirst += plant.kind.transpiration * time;
                //     plant_updated = true;
                // }

                plant_updated = true;

                plant.growth = 3.5;

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
