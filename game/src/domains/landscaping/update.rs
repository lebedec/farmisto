use crate::landscaping::{Landscaping, LandscapingDomain, LAND_HEIGHT, LAND_WIDTH};
use rand::Rng;

impl LandscapingDomain {
    pub fn update(&mut self, time: f32, mut random: impl Rng) -> Vec<Landscaping> {
        let mut events = vec![];
        let mut need_update = false;
        self.lands_update += time;
        if self.lands_update > self.lands_update_interval {
            self.lands_update -= self.lands_update_interval;
            need_update = true;
        }
        for land in self.lands.values_mut() {
            for y in 0..LAND_HEIGHT {
                for x in 0..LAND_WIDTH {
                    let _capacity = land.moisture_capacity[y][x] as f32 / 255.0;
                    let moisture = land.moisture[y][x] as f32 / 255.0;
                    let heat = random.gen_range(0.01..0.11);
                    land.moisture[y][x] = ((moisture - heat * time).max(0.0) * 255.0) as u8;
                }
            }
            if need_update {
                events.push(Landscaping::MoistureUpdate {
                    land: land.id,
                    moisture: land.moisture,
                });
            }
        }
        events
    }
}
