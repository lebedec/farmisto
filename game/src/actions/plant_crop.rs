use crate::api::{ActionError, Event};
use crate::inventory::FunctionsQuery;
use crate::model::{Activity, CropKey, Farmer, Farmland};
use crate::{occur, position_of, Game};

impl Game {
    pub(crate) fn plant_crop(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let key = item.kind.functions.as_seeds(CropKey)?;
        let kind = self.known.crops.get(key)?;
        let position = position_of(tile);
        let decrease_item = self.inventory.decrease_item(farmer.hands)?;
        let (barrier, sensor, create_barrier_sensor) = self.physics.create_barrier_sensor(
            farmland.space,
            &kind.barrier,
            &kind.sensor,
            position,
            false,
        )?;
        let (plant, create_plant) = self
            .planting
            .create_plant(farmland.soil, &kind.plant, 0.0)?;
        let events = occur![
            decrease_item(),
            create_barrier_sensor(),
            create_plant(),
            self.appear_crop(kind.id, barrier, sensor, plant)?,
        ];
        Ok(events)
    }
}
