use crate::api::{ActionError, Event};
use crate::model::Activity::Idle;
use crate::model::{Activity, Equipment, Farmer, Purpose};
use crate::{occur, Game};

impl Game {
    pub(crate) fn use_equipment(
        &mut self,
        farmer: Farmer,
        equipment: Equipment,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Idle)?;
        let events = match equipment.purpose {
            Purpose::Moisture { .. } => {
                vec![]
            }
            Purpose::Tethering { tether } => {
                let activity = Activity::Tethering2 { tether };
                vec![self.universe.change_activity(farmer, activity)]
            }
        };
        Ok(occur![events,])
    }
}
