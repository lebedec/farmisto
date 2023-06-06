use crate::building::{Building, BuildingDomain, BuildingError, Marker, Stake, SurveyorId};
use crate::math::Tile;

impl BuildingDomain {
    pub fn survey(
        & mut self,
        surveyor: SurveyorId,
        marker: Marker,
        cell: Tile
    ) -> Result<(usize, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let id = surveyor.stake_id + 1;
        let stake = Stake {
            id,
            marker,
            cell,
        };
        let command = move || {
            surveyor.stake_id = id;
            surveyor.surveying.push(stake);
            vec![]
        };
        Ok((id, command))
    }
}
