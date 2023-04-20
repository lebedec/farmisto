use crate::landscaping::{
    LandId, Landscaping, LandscapingDomain, LandscapingError, Place, Surface,
};

impl LandscapingDomain {
    pub fn fill_basin(
        &mut self,
        id: LandId,
        place: Place,
    ) -> Result<impl FnOnce() -> Vec<Landscaping> + '_, LandscapingError> {
        let land = self.get_land_mut(id)?;
        land.ensure_surface(place, Surface::BASIN)?;
        let command = move || {
            let [x, y] = place;
            land.surface[y][x] = Surface::PLAINS;
            land.moisture_capacity[y][x] = 127;
            vec![
                Landscaping::SurfaceUpdate {
                    land: land.id,
                    surface: land.surface,
                },
                Landscaping::MoistureCapacityUpdate {
                    land: land.id,
                    moisture_capacity: land.moisture_capacity,
                },
            ]
        };
        Ok(command)
    }
}
