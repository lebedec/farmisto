use crate::building::Building::SurveyorModeChanged;
use crate::building::{Building, BuildingDomain, BuildingError, Stake, Surveyor, SurveyorId};

impl BuildingDomain {
    pub fn set_surveyor_mode(
        &mut self,
        surveyor: SurveyorId,
        mode: u8,
    ) -> Result<impl FnOnce() -> Vec<Building> + '_, BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let command = move || {
            surveyor.mode = mode % Surveyor::MODES.len() as u8;
            vec![SurveyorModeChanged {
                id: surveyor.id,
                mode: surveyor.mode,
            }]
        };
        Ok(command)
    }
}
