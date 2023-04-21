use crate::landscaping::{Land, LandscapingError, Place};
use crate::math::TileMath;

impl Land {
    pub fn get_moisture(&self, place: Place) -> Result<f32, LandscapingError> {
        let [x, y] = place;
        if y < self.kind.height && x < self.kind.width {
            let index = place.fit(self.kind.width);
            Ok(self.moisture[index])
        } else {
            Err(LandscapingError::OutOfLand { place, id: self.id })
        }
    }

    pub fn get_moisture_capacity(&self, place: Place) -> Result<f32, LandscapingError> {
        let [x, y] = place;
        if y < self.kind.height && x < self.kind.width {
            let index = place.fit(self.kind.width);
            Ok(self.moisture_capacity[index])
        } else {
            Err(LandscapingError::OutOfLand { place, id: self.id })
        }
    }

    pub fn ensure_surface(&self, place: Place, expected: u8) -> Result<u8, LandscapingError> {
        let [x, y] = place;
        if y < self.kind.height && x < self.kind.width {
            let index = place.fit(self.kind.width);
            let actual = self.surface[index];
            if actual != expected {
                Err(LandscapingError::InvalidLandSurface {
                    id: self.id,
                    actual,
                    expected,
                })
            } else {
                Ok(actual)
            }
        } else {
            Err(LandscapingError::OutOfLand { place, id: self.id })
        }
    }
}
