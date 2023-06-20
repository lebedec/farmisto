use crate::building::{
    Building, BuildingDomain, BuildingError, Marker, Stake, Structure, Surveyor, SurveyorId,
};
use crate::math::Tile;

impl BuildingDomain {
    pub fn construct(
        &mut self,
        surveyor: SurveyorId,
        cell: Tile,
    ) -> Result<(usize, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let id = surveyor.stake_id + 1;
        let structure = match surveyor.mode {
            Surveyor::MODE_WALL => Structure::Wall,
            Surveyor::MODE_DOOR => Structure::Door,
            Surveyor::MODE_WINDOW => Structure::Window,
            _ => return Err(BuildingError::SurveyorMarkerNotFound),
        };
        let marker = Marker::Construction(structure);
        let stake = Stake { id, marker, cell };
        let command = move || {
            surveyor.stake_id = id;
            surveyor.surveying.push(stake);
            vec![]
        };
        Ok((id, command))
    }

    pub fn reconstruct(
        &mut self,
        surveyor: SurveyorId,
        cell: Tile,
    ) -> Result<(usize, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let id = surveyor.stake_id + 1;
        let structure = match surveyor.mode {
            Surveyor::MODE_WALL => Structure::Wall,
            Surveyor::MODE_DOOR => Structure::Door,
            Surveyor::MODE_WINDOW => Structure::Window,
            _ => return Err(BuildingError::SurveyorMarkerNotFound),
        };
        let marker = Marker::Reconstruction(structure);
        let stake = Stake { id, marker, cell };
        let command = move || {
            surveyor.stake_id = id;
            surveyor.surveying.push(stake);
            vec![]
        };
        Ok((id, command))
    }

    pub fn deconstruct(
        &mut self,
        surveyor: SurveyorId,
        cell: Tile,
    ) -> Result<(usize, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        let surveyor = self.get_surveyor_mut(surveyor)?;
        let id = surveyor.stake_id + 1;
        let marker = Marker::Deconstruction;
        let stake = Stake { id, marker, cell };
        let command = move || {
            surveyor.stake_id = id;
            surveyor.surveying.push(stake);
            vec![]
        };
        Ok((id, command))
    }
}
