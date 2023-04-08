use crate::{Game, occur};
use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::model::{Activity, Farmer};

impl Game {
    pub(crate) fn put_item(
        &mut self,
        farmer: Farmer,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let hands = self.inventory.get_container(farmer.hands)?;
        let is_last_item = hands.items.len() <= 1;
        let transfer = self.inventory.pop_item(farmer.hands, container)?;
        // TODO: check tags
        // TODO: quantity merge
        // TODO: capacity check
        let activity = if is_last_item {
            self.universe.change_activity(farmer, Activity::Idle)
        } else {
            vec![]
        };
        let events = occur![transfer(), activity,];
        Ok(events)
    }
}
