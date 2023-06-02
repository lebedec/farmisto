use rand::rngs::ThreadRng;
use rand::Rng;

use crate::landscaping::{
    LandId, Landscaping, LandscapingDomain, LandscapingError, Place, Surface,
};
use crate::math::{ArrayIndex, TileMath, VectorMath};

impl LandscapingDomain {
    pub fn pour_water(
        &mut self,
        id: LandId,
        place: Place,
        volume: f32,
        spread: u32,
        mut random: ThreadRng,
    ) -> Result<impl FnOnce() -> Vec<Landscaping> + '_, LandscapingError> {
        let spread = spread as i32;
        let land = self.get_land_mut(id)?;
        land.ensure_surface(place, Surface::PLAINS)?;

        let command = move || {
            let center = place.position();
            let [x, y] = place;
            for sy in -spread..=spread {
                for sx in -spread..=spread {
                    let [sx, sy] = [x as i32 + sx, y as i32 + sy];
                    if sx < 0 || sy < 0 || sx >= 128 || sy >= 128 {
                        continue;
                    }
                    let x = sx as usize;
                    let y = sy as usize;
                    let place = [x, y];
                    let distance = place.position().distance(center);
                    let place = place.fit(land.kind.width);
                    let moisture = land.moisture[place];
                    let moisture_capacity = land.moisture_capacity[place];
                    let mut factor = if distance == 1.0 {
                        1.0
                    } else {
                        let step = 1.0 / (1 + spread) as f32;
                        (1.0 - step * distance).max(0.0)
                    };
                    factor += random.gen_range(0.0..0.1);
                    let volume = volume * factor;
                    let moisture = (moisture + volume).min(moisture_capacity);
                    land.moisture[place] = moisture;
                }
            }
            vec![]
        };
        Ok(command)
    }
}
