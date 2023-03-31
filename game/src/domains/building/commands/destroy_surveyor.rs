use crate::building::Building::SurveyorDestroyed;
use crate::building::{Building, BuildingDomain, BuildingError, SurveyorId};

impl BuildingDomain {
    pub fn destroy_surveyor<'operation>(
        &'operation mut self,
        id: SurveyorId,
    ) -> Result<impl FnOnce() -> Vec<Building> + 'operation, BuildingError> {
        let index = self.index_surveyor(id)?;
        let operation = move || {
            self.surveyors.remove(index);
            vec![SurveyorDestroyed { id }]
        };
        Ok(operation)
    }
}
