use crate::api::{ActionError, Event};
use crate::building::SurveyorId;
use crate::model::{Activity, Equipment, EquipmentKey, Farmer, Purpose, UniverseError};
use crate::physics::BarrierId;
use crate::{emit, occur, Game};

impl Game {
    pub(crate) fn toggle_surveying_option(
        &mut self,
        farmer: Farmer,
        mode: u8,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        let theodolite = activity.as_surveying()?;
        let set_surveyor_mode = self.building.set_surveyor_mode(theodolite.surveyor, mode)?;
        emit![set_surveyor_mode()]
    }
}
