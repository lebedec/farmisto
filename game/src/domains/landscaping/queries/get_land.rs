use crate::landscaping::{Land, LandId, LandscapingDomain, LandscapingError};

impl LandscapingDomain {
    pub fn get_land(&self, id: LandId) -> Result<&Land, LandscapingError> {
        self.lands
            .get(&id)
            .ok_or(LandscapingError::LandNotFound { id })
    }

    pub fn get_land_mut(&mut self, id: LandId) -> Result<&mut Land, LandscapingError> {
        self.lands
            .get_mut(&id)
            .ok_or(LandscapingError::LandNotFound { id })
    }
}
