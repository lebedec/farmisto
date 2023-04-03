use crate::planting::Planting::PlantUpdated;
use crate::planting::{Planting, PlantingDomain};

impl PlantingDomain {
    pub fn update_impact(&mut self) {}

    pub fn update(&mut self, time: f32) -> Vec<Planting> {
        let mut events = vec![];
        for land in self.soils.iter_mut() {
            for row in land.map.iter_mut() {
                for cell in row.iter_mut() {
                    let [capacity, moisture] = *cell;
                    *cell = [capacity, (moisture - 0.1 * time).max(0.0)];
                }
            }
            events.push(Planting::SoilChanged {
                soil: land.id,
                map: land.map.clone(),
            })
        }
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
                    plant_updated = true;
                }

                if plant.thirst <= 1.0 {
                    plant.thirst += plant.kind.transpiration * time;
                    plant_updated = true;
                }

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
