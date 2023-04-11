use crate::building::{BuildingDomain, BuildingError, GridId, Surveyor, SurveyorId};

impl BuildingDomain {

    pub fn find_surveyor_mut(
        &mut self,
        grid: GridId,
        cell: [usize; 2],
    ) -> Result<&mut Surveyor, BuildingError> {
        self.surveyors
            .iter_mut()
            .find(|surveyor| {
                surveyor.grid == grid && surveyor.surveying.iter().any(|stake| stake.cell == cell)
            })
            .ok_or(BuildingError::SurveyorMarkerNotFound)
    }

    pub fn index_surveyor2(
        &mut self,
        grid: GridId,
        cell: [usize; 2],
    ) -> Result<usize, BuildingError> {
        self.surveyors
            .iter_mut()
            .position(|surveyor| {
                surveyor.grid == grid && surveyor.surveying.iter().any(|stake| stake.cell == cell)
            })
            .ok_or(BuildingError::SurveyorMarkerNotFound)
    }

    #[inline]
    pub fn get_surveyor(&self, id: SurveyorId) -> Result<&Surveyor, BuildingError> {
        self.surveyors
            .iter()
            .find(|surveyor| surveyor.id == id)
            .ok_or(BuildingError::SurveyorNotFound { id })
    }

    #[inline]
    pub fn get_surveyor_mut(&mut self, id: SurveyorId) -> Result<&mut Surveyor, BuildingError> {
        self.surveyors
            .iter_mut()
            .find(|surveyor| surveyor.id == id)
            .ok_or(BuildingError::SurveyorNotFound { id })
    }

    #[inline]
    pub fn index_surveyor(&self, id: SurveyorId) -> Result<usize, BuildingError> {
        self.surveyors
            .iter()
            .position(|surveyor| surveyor.id == id)
            .ok_or(BuildingError::SurveyorNotFound { id })
    }
}