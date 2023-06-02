use rand::thread_rng;

use crate::api::{ActionError, Event};
use crate::inventory::FunctionsQuery;
use crate::math::{Tile, TileMath};
use crate::model::{Activity, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn pour_water(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        place: Tile,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        self.ensure_target_reachable(farmer.body, place.position())?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let nozzle = item.kind.functions.as_moistener()?;
        let random = thread_rng();
        let pour_water = self.landscaping.pour_water(
            farmland.land,
            place,
            nozzle.pressure,
            nozzle.spread,
            random,
        )?;
        let events = occur![pour_water(),];
        Ok(events)
    }
}
