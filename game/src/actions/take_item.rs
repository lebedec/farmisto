use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::model::{Activity, Farmer};
use crate::{occur, Game};

impl Game {
    pub(crate) fn take_item(
        &mut self,
        farmer: Farmer,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let pop_item = self.inventory.pop_item(container, farmer.hands)?;
        // TODO: check tags
        // TODO: quantity merge
        // TODO: capacity check
        let events = occur![
            pop_item(),
            self.universe.change_activity(farmer, Activity::Usage),
        ];
        Ok(events)
    }
}
