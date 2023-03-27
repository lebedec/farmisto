use crate::building::{Building, BuildingDomain, BuildingError, Stake, SurveyorId};

impl BuildingDomain {
    pub fn unmark<'operation>(
        &'operation mut self,
        surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Result<impl FnOnce() -> Vec<Building> + 'operation, BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let operation = move || {
            let index = surveyor
                .surveying
                .iter()
                .position(|marker| marker.cell == cell)
                .unwrap();
            surveyor.surveying.remove(index);
            vec![]
        };
        Ok(operation)
    }
}
