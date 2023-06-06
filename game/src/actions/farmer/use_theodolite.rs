use crate::api::{ActionError, Event};
use crate::model::Activity::Idle;
use crate::model::{Activity, Farmer, Theodolite};
use crate::{emit, occur, Game};

impl Game {
    pub(crate) fn use_theodolite(
        &mut self,
        farmer: Farmer,
        theodolite: Theodolite,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Idle)?;
        let destination = self.physics.get_barrier(theodolite.barrier)?.position;
        self.ensure_target_reachable(farmer.body, destination)?;
        let activity = Activity::Surveying2 {
            theodolite,
            selection: 0,
        };
        emit![self.universe.change_activity(farmer, activity)]
    }
}
