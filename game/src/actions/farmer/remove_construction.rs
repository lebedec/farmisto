use crate::api::{ActionError, Event};
use crate::model::{Construction, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn remove_construction(
        &mut self,
        _farmer: Farmer,
        _farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let destroy_container = self
            .inventory
            .destroy_containers(vec![construction.container], false)?;
        let destroy_marker = self
            .building
            .unmark(construction.surveyor, construction.stake)?;
        let events = occur![
            destroy_container(),
            destroy_marker(),
            self.universe.vanish_construction(construction),
        ];
        Ok(events)
    }
}
