use crate::api::{ActionError, Event};
use crate::model::Farmer;
use crate::working::DeviceId;
use crate::{occur, Game};

impl Game {
    pub(crate) fn toggle_generic_device(
        &mut self,
        farmer: Farmer,
        device: DeviceId,
    ) -> Result<Vec<Event>, ActionError> {
        let toggle_device = self.working.toggle_device(device)?;
        let events = occur![toggle_device(),];
        Ok(events)
    }
}
