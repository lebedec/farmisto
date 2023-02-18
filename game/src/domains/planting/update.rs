use crate::planting::{Planting, PlantingDomain};

impl PlantingDomain {
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
        events
    }
}
