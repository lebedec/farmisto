use crate::api::{ActionError, Event};
use crate::model::{Activity, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn cancel_assembly(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        let assembly = activity.as_assembling()?;
        let cancel_placement = self.assembling.cancel_placement(assembly.placement)?;
        self.universe.change_activity(farmer, Activity::Usage);
        let events = occur![
            cancel_placement(),
            self.universe.vanish_assembly(assembly),
            self.universe.change_activity(farmer, Activity::Usage),
        ];
        Ok(events)
    }
}
