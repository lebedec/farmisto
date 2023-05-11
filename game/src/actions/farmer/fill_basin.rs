use crate::api::{ActionError, Event};
use crate::inventory::{ContainerId, FunctionsQuery, Item, ItemId};
use crate::landscaping::Surface;
use crate::math::{Tile, TileMath};
use crate::model::{Activity, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn fill_basin(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        place: Tile,
    ) -> Result<Vec<Event>, ActionError> {
        // self.ensure_target_reachable(farmer.body, place.to_position())?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        item.kind.functions.as_stone()?;
        let land = self.landscaping.get_land(farmland.land)?;
        land.ensure_surface(place, Surface::BASIN)?;

        let use_item = self.inventory.use_items_from(farmer.hands)?;
        let fill_basin = self.landscaping.fill_basin(farmland.land, place)?;
        let destroy_hole = self.physics.destroy_hole(farmland.space, place)?;

        let events = occur![use_item(), fill_basin(), destroy_hole(),];

        Ok(events)
    }
}
