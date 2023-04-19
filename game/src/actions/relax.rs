use crate::api::{ActionError, Event};
use crate::model::{Activity, Farmer, Farmland, Rest};
use crate::{occur, Game};

impl Game {
    pub(crate) fn relax(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        rest: Rest,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        let destination = self.physics.get_barrier(rest.barrier)?.position;
        self.ensure_target_reachable(farmland.space, farmer, destination)?;
        let rest_kind = self.known.rests.get(rest.key)?;
        let events = self.universe.change_activity(
            farmer,
            Activity::Resting {
                comfort: rest_kind.comfort,
            },
        );
        let events = occur![events,];
        Ok(events)
    }
}
