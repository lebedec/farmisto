use rand::Rng;

use crate::landscaping::{Landscaping, LandscapingDomain};
use crate::math::TileMath;

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
            for y in 0..land.kind.height {
                for x in 0..land.kind.width {
                    let place = [x, y].fit(land.kind.width);
                    let moisture = land.moisture[place];
                    let heat = random.gen_range(0.01..0.11);
                    land.moisture[place] = (moisture - heat * time).max(0.0);
                }
            }
            if need_update {
                // events.push(Landscaping::MoistureUpdate {
                //     land: land.id,
                //     moisture: land.moisture,
                // });
            }
        }
        events
    }
}
