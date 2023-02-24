use crate::planting::Planting::PlantUpdated;
use crate::planting::{Planting, PlantingDomain};

impl PlantingDomain {
    pub fn update_impact(&mut self) {}

    pub fn update(&mut self, time: f32) -> Vec<Planting> {
        let mut events = vec![];
        for land in self.lands.iter_mut() {
            for row in land.map.iter_mut() {
                for cell in row.iter_mut() {
                    let [capacity, moisture] = *cell;
                    *cell = [capacity, (moisture - 0.1 * time).max(0.0)];
                }
            }
            events.push(Planting::LandChanged {
                land: land.id,
                map: land.map.clone(),
            })
        }
        for plants in &mut self.plants {
            for plant in plants.iter_mut() {
                if plant.impact.abs() > 0.001 {
                    let delta = time * plant.kind.flexibility;
                    plant.impact = if delta > plant.impact.abs() {
                        0.0
                    } else {
                        plant.impact - (plant.impact.signum() * time * plant.kind.flexibility)
                    };
                    events.push(PlantUpdated {
                        id: plant.id,
                        impact: plant.impact,
                    })
                }
            }
        }
        events
    }
}