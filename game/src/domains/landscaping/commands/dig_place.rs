use crate::landscaping::{
    LandId, Landscaping, LandscapingDomain, LandscapingError, Place, Surface,
};

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
            let [x, y] = place;
            if capacity == 1.0 {
                land.surface[y][x] = Surface::BASIN;
                vec![Landscaping::SurfaceUpdate {
                    land: land.id,
                    surface: land.surface,
                }]
            } else {
                let capacity = (capacity + quality).min(1.0) * 255.0;
                land.moisture_capacity[y][x] = capacity as u8;
                vec![Landscaping::MoistureCapacityUpdate {
                    land: land.id,
                    moisture_capacity: land.moisture_capacity,
                }]
            }
        };
        Ok(command)
    }
}
