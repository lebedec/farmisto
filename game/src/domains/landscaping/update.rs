use std::collections::HashSet;

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::landscaping::{Land, LandId, Landscaping, LandscapingDomain, LandscapingError, Surface};
use crate::math::TileOffsetMath;

impl LandscapingDomain {
    pub fn request_consumption(
        &mut self,
        land: LandId,
        place: usize,
        expected: f32,
    ) -> Result<f32, LandscapingError> {
        let land = self.get_land_mut(land)?;
        let moisture = land.moisture[place];
        let delta = expected.min(moisture);
        land.moisture[place] -= delta;
        Ok(delta)
    }

    pub fn update(&mut self, time: f32, random: ThreadRng) -> Vec<Landscaping> {
        for land in self.lands.values_mut() {
            LandscapingDomain::drain(land, time, random.clone());
            LandscapingDomain::disperse_moisture_from_water(land, time, 0.75);
        }
        vec![]
    }

    pub fn drain(land: &mut Land, time: f32, mut random: impl Rng) {
        for moisture in land.moisture.iter_mut() {
            let heat = random.gen_range(0.0025..0.0035);
            *moisture = (*moisture - heat * time).max(0.0);
        }
    }

    pub fn disperse_moisture_from_water(land: &mut Land, time: f32, pressure: f32) {
        let mut source = HashSet::new();
        let mut flow = Vec::with_capacity(128);
        for (place, surface) in land.surface.iter().enumerate() {
            if surface == &Surface::BASIN {
                source.insert(place);
                let directions = [
                    [-1, -1],
                    [-1, 0],
                    [-1, 1],
                    [0, 1],
                    [1, 1],
                    [1, 0],
                    [1, -1],
                    [0, -1],
                ];
                let directions = directions.map(|offset| offset.fit_offset(land.kind.width));
                let mut fluid = 1;
                for direction in directions {
                    let neighbor = place as isize - direction;
                    if neighbor < 0 {
                        continue;
                    }
                    let neighbor = neighbor as usize;
                    if neighbor >= land.surface.len() {
                        continue;
                    }
                    if land.surface[neighbor] == Surface::BASIN {
                        fluid += 1;
                    }
                }
                let fluid = (fluid as f32 / 8.0).min(1.0);
                flow.push((place, fluid));
            }
        }
        let directions = [[-1, 0], [1, 0], [0, -1], [0, 1]];
        let directions = directions.map(|offset| offset.fit_offset(land.kind.width));
        while let Some((place, fluid)) = flow.pop() {
            let volume = fluid * time;
            let moisture = land.moisture[place];
            let moisture_capacity = land.moisture_capacity[place];
            land.moisture[place] = (moisture + volume).min(moisture_capacity);
            for direction in directions {
                let next_place = place as isize - direction;
                if next_place < 0 {
                    continue;
                }
                let next_place = next_place as usize;
                if next_place >= land.surface.len() || source.contains(&next_place) {
                    continue;
                }
                let moisture_capacity = land.moisture_capacity[next_place];
                let fluid = moisture_capacity * pressure * fluid;
                if fluid > 0.001 {
                    source.insert(next_place);
                    flow.insert(0, (next_place, fluid))
                }
            }
        }
    }

    pub fn disperse_moisture(_land: &mut Land) {}
}
