use crate::building::{Building, BuildingDomain, BuildingError, Stake, SurveyorId};

impl BuildingDomain {
    pub fn survey<'operation>(
        &'operation mut self,
        surveyor: SurveyorId,
        stake: Stake,
    ) -> Result<impl FnOnce() -> Vec<Building> + 'operation, BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let operation = move || {
            surveyor.surveying.push(stake);
            vec![]
        };
        Ok(operation)
    }
}
