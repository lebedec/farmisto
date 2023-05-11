use crate::api::{ActionError, Event};
use crate::model::Farmer;
use crate::working::DeviceId;
use crate::{occur, Game};

impl Game {
    pub(crate) fn repair_generic_device(
        &mut self,
        farmer: Farmer,
        device: DeviceId,
    ) -> Result<Vec<Event>, ActionError> {
        let repair_device = self.working.repair_device(device)?;
        let events = occur![repair_device(),];
        Ok(events)
    }
}
