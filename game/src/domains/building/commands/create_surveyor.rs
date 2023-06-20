use crate::building::Building::SurveyorCreated;
use crate::building::{
    Building, BuildingDomain, BuildingError, GridId, Surveyor, SurveyorId, SurveyorKind,
};
use crate::collections::Shared;

impl BuildingDomain {
    pub fn create_surveyor(
        &mut self,
        grid: GridId,
        kind: Shared<SurveyorKind>,
    ) -> Result<(SurveyorId, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        let id = SurveyorId(self.surveyors_sequence + 1);
        let surveyor = Surveyor {
            id,
            grid,
            stake_id: 0,
            surveying: vec![],
            kind,
            mode: 0,
        };
        let command = move || {
            let events = vec![SurveyorCreated {
                id: surveyor.id,
                grid: surveyor.grid,
            }];
            self.surveyors_sequence += 1;
            self.surveyors.push(surveyor);
            events
        };
        Ok((id, command))
    }
}
