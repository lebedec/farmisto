use crate::api::{ActionError, Event};
use crate::building::{Marker, Stake, SurveyorId};
use crate::inventory::ContainerId;
use crate::model::{Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn survey(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        surveyor: SurveyorId,
        cell: [usize; 2],
        marker: Marker,
    ) -> Result<Vec<Event>, ActionError> {
        let stake = Stake { marker, cell };
        let survey = self.building.survey(surveyor, stake)?;
        let container_kind = self.known.containers.find("<construction>")?;
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let create_container = self
            .inventory
            .add_empty_container(container, &container_kind)?;
        let events = occur![
            survey(),
            create_container(),
            self.appear_construction(container, farmland.grid, surveyor, marker, cell),
        ];
        Ok(events)
    }
}
