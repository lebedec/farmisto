use crate::building::{BuildingError, Stake, Surveyor};

impl Surveyor {
    pub fn get_stake(&self, id: usize) -> Result<&Stake, BuildingError> {
        self.surveying
            .iter()
            .find(|stake| stake.id == id)
            .ok_or(BuildingError::StakeNotFound { id })
    }

    pub fn index_stake(&self, id: usize) -> Result<usize, BuildingError> {
        self.surveying
            .iter()
            .position(|stake| stake.id == id)
            .ok_or(BuildingError::StakeNotFound { id })
    }
}
