use crate::landscaping::{LandId, Landscaping, LandscapingDomain, LandscapingError};

impl LandscapingDomain {
    pub fn plow_place(
        &mut self,
        id: LandId,
        place: [usize; 2],
        quality: f32,
    ) -> Result<impl FnOnce() -> Vec<Landscaping> + '_, LandscapingError> {
        let land = self.get_land_mut(id)?;
        let capacity = land.get_moisture_capacity(place)?;
        let command = move || {
            let [x, y] = place;
            let capacity = (capacity + quality).min(1.0) * 255.0;
            land.moisture_capacity[y][x] = capacity as u8;
            vec![Landscaping::MoistureCapacityUpdate {
                land: land.id,
                moisture_capacity: land.moisture_capacity,
            }]
        };
        Ok(command)
    }
}
