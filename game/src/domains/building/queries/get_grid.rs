use crate::building::{BuildingDomain, BuildingError, Grid, GridId};

impl BuildingDomain {
    #[inline]
    pub fn get_grid(&self, id: GridId) -> Result<&Grid, BuildingError> {
        self.grids
            .iter()
            .find(|grid| grid.id == id)
            .ok_or(BuildingError::GridNotFound { id })
    }

    #[inline]
    pub fn index_grid(&mut self, id: GridId) -> Result<usize, BuildingError> {
        self.grids
            .iter_mut()
            .position(|grid| grid.id == id)
            .ok_or(BuildingError::GridNotFound { id })
    }

    #[inline]
    pub fn get_mut_grid(&mut self, id: GridId) -> Result<&mut Grid, BuildingError> {
        self.grids
            .iter_mut()
            .find(|grid| grid.id == id)
            .ok_or(BuildingError::GridNotFound { id })
    }
}