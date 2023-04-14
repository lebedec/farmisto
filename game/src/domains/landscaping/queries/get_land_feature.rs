use crate::landscaping::{Land, LandscapingError, Place, LAND_HEIGHT, LAND_WIDTH};

impl Land {
    pub fn get_moisture(&self, place: Place) -> Result<f32, LandscapingError> {
        let [x, y] = place;
        if y < LAND_HEIGHT && x < LAND_WIDTH {
            Ok(self.moisture[y][x] as f32 / 255.0)
        } else {
            Err(LandscapingError::OutOfLand { place, id: self.id })
        }
    }

    pub fn get_moisture_capacity(&self, place: Place) -> Result<f32, LandscapingError> {
        let [x, y] = place;
        if y < LAND_HEIGHT && x < LAND_WIDTH {
            Ok(self.moisture_capacity[y][x] as f32 / 255.0)
        } else {
            Err(LandscapingError::OutOfLand { place, id: self.id })
        }
    }
}
