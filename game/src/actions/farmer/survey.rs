use crate::api::{ActionError, Event};
use crate::building::{Marker, Stake, SurveyorId};
use crate::inventory::ContainerId;
use crate::model::{Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn construct(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let (stake, survey) = self.building.construct(surveyor, cell)?;
        let container_kind = self.known.containers.find("<construction>")?;
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let create_container = self
            .inventory
            .add_empty_container(container, &container_kind)?;
        let events = occur![
            survey(),
            create_container(),
            self.appear_construction(container, farmland.grid, surveyor, stake)?,
        ];
        Ok(events)
    }

    pub(crate) fn reconstruct(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let (stake, survey) = self.building.reconstruct(surveyor, cell)?;
        let container_kind = self.known.containers.find("<construction>")?;
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let create_container = self
            .inventory
            .add_empty_container(container, &container_kind)?;
        let events = occur![
            survey(),
            create_container(),
            self.appear_construction(container, farmland.grid, surveyor, stake)?,
        ];
        Ok(events)
    }

    pub(crate) fn deconstruct(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let (stake, survey) = self.building.deconstruct(surveyor, cell)?;
        let container_kind = self.known.containers.find("<construction>")?;
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let create_container = self
            .inventory
            .add_empty_container(container, &container_kind)?;
        let events = occur![
            survey(),
            create_container(),
            self.appear_construction(container, farmland.grid, surveyor, stake)?,
        ];
        Ok(events)
    }
}
