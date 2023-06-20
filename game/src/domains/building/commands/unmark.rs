use crate::building::{Building, BuildingDomain, BuildingError, SurveyorId};

impl BuildingDomain {
    pub fn unmark(
        &mut self,
        surveyor: SurveyorId,
        stake: usize,
    ) -> Result<impl FnOnce() -> Vec<Building> + '_, BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let index = surveyor.index_stake(stake)?;
        let command = move || {
            surveyor.surveying.remove(index);
            vec![]
        };
        Ok(command)
    }
}
