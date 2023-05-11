use crate::api::{ActionError, Event};
use crate::model::{Crop, Farmer};
use crate::{occur, Game};

impl Game {
    pub(crate) fn water_crop(
        &mut self,
        _farmer: Farmer,
        crop: Crop,
    ) -> Result<Vec<Event>, ActionError> {
        let water_plant = self.planting.water_plant(crop.plant, 0.5)?;
        let events = occur![water_plant(),];
        Ok(events)
    }
}
