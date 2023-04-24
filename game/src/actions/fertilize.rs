use rand::thread_rng;

use crate::api::{ActionError, Event};
use crate::inventory::FunctionsQuery;
use crate::math::{Tile, TileMath};
use crate::model::{Activity, Farmer, Farmland, Purpose, PurposeDescription};
use crate::{occur, Game};

impl Game {
    pub(crate) fn fertilize(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: Tile,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        self.ensure_target_reachable(farmland.space, farmer, tile.position())?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let quality = item.kind.functions.as_fertilizer()?;
        let decrease_item = self.inventory.decrease_item(farmer.hands)?;
        let fertilize = self.planting.fertilize(farmland.soil, tile, quality)?;
        let events = occur![decrease_item(), fertilize(),];
        Ok(events)
    }
}
