use crate::landscaping::{
    LandId, Landscaping, LandscapingDomain, LandscapingError, Place, Surface,
};
use crate::math::{Array2D, TileMath};

impl LandscapingDomain {
    pub fn fill_basin(
        &mut self,
        id: LandId,
        place: Place,
    ) -> Result<impl FnOnce() -> Vec<Landscaping> + '_, LandscapingError> {
        let land = self.get_land_mut(id)?;
        land.ensure_surface(place, Surface::BASIN)?;
        let command = move || {
            let place = place.fit(land.kind.width);
            land.surface[place] = Surface::PLAINS;
            land.moisture_capacity[place] = 0.5;
            vec![]
        };
        Ok(command)
    }
}
