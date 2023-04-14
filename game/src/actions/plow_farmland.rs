use crate::api::{ActionError, Event};
use crate::inventory::{ContainerId, Item, ItemId};
use crate::math::{Tile, TileMath};
use crate::model::{Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn plow_farmland(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        place: Tile,
    ) -> Result<Vec<Event>, ActionError> {
        self.ensure_target_reachable(farmland.space, farmer, place.to_position())?;
        let quality = 0.05;
        let land = self.landscaping.get_land(farmland.land)?;
        let capacity = land.get_moisture_capacity(place)?;
        let is_stones_gathered = capacity >= 0.55 && capacity < 0.6;
        let plow_place = self.landscaping.plow_place(farmland.land, place, quality)?;
        let events = if is_stones_gathered {
            let barrier_kind = self.known.barriers.find("<drop>").unwrap();
            let (barrier, create_barrier) = self.physics.create_barrier(
                farmland.space,
                barrier_kind,
                place.to_position(),
                true,
                false,
            )?;

            let item_kind = self.known.items.find("stones")?;
            let container_kind = self.known.containers.find("<drop>").unwrap();
            let container = self.inventory.containers_id.introduce().one(ContainerId);
            let item = self.inventory.items_id.introduce().one(ItemId);
            let items = vec![Item {
                id: item,
                kind: item_kind,
                container,
                quantity: 1,
            }];
            let create_stones = self
                .inventory
                .add_container(container, &container_kind, items)?;

            occur![
                plow_place(),
                create_barrier(),
                create_stones(),
                self.appear_stack(container, barrier),
            ]
        } else {
            occur![plow_place(),]
        };
        Ok(events)
    }
}
