use crate::landscaping::{
    LandId, Landscaping, LandscapingDomain, LandscapingError, Place, Surface,
};
use crate::math::{Array2D, TileMath};

impl LandscapingDomain {
    pub fn dig_place(
        &mut self,
        id: LandId,
        place: Place,
        quality: f32,
    ) -> Result<impl FnOnce() -> Vec<Landscaping> + '_, LandscapingError> {
        let land = self.get_land_mut(id)?;
        let capacity = land.get_moisture_capacity(place)?;
        land.ensure_surface(place, Surface::PLAINS)?;
        let command = move || {
            let place = place.fit(land.kind.width);
            if capacity == 1.0 {
                land.surface[place] = Surface::BASIN;
                vec![]
            } else {
                let capacity = (capacity + quality).min(1.0);
                land.moisture_capacity[place] = capacity;
                vec![]
            }
        };
        Ok(command)
    }
}
