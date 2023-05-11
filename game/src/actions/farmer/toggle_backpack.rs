use crate::api::{ActionError, Event};
use crate::{Game, occur};
use crate::model::{Farmer, Activity};

impl Game {

    pub(crate) fn toggle_backpack(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let backpack_empty = self
            .inventory
            .get_container(farmer.backpack)?
            .items
            .is_empty();
        let hands_empty = self.inventory.get_container(farmer.hands)?.items.is_empty();
        let mut events = vec![];
        if hands_empty && !backpack_empty {
            let transfer = self.inventory.pop_item(farmer.backpack, farmer.hands)?;
            events = occur![
                transfer(),
                self.universe.change_activity(farmer, Activity::Usage),
            ];
        }
        if !hands_empty && backpack_empty {
            let transfer = self.inventory.pop_item(farmer.hands, farmer.backpack)?;
            events = occur![
                transfer(),
                self.universe.change_activity(farmer, Activity::Idle),
            ];
        }
        Ok(events)
    }
}