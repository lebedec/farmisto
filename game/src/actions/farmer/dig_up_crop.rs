use crate::api::{ActionError, Event};
use crate::inventory::{ContainerId, FunctionsQuery, Item, ItemId};
use crate::model::{Crop, Farmer, Farmland};
use crate::{occur, Game};
use log::info;

impl Game {
    pub(crate) fn dig_up_crop(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        crop: Crop,
    ) -> Result<Vec<Event>, ActionError> {
        let item = self.inventory.get_container_item(farmer.hands)?;
        item.kind.functions.as_shovel()?;
        let position = self.physics.get_barrier(crop.barrier)?.position;
        self.ensure_target_reachable(farmer.body, position)?;
        let crop_kind = self.known.crops.get(crop.key)?;

        // TODO: transactional
        let destroy_plant_barrier = self.physics.destroy_barrier(crop.barrier)?();
        self.physics.destroy_sensor(crop.sensor)?();

        let (residue, destroy_plant) = self.planting.destroy_plant(crop.plant)?;

        let barrier_kind = self.known.barriers.find("<drop>").unwrap();
        let (barrier, create_barrier) =
            self.physics
                .create_barrier(farmland.space, barrier_kind, position, true, false)?;

        let quantity = 1 + (9.0 * residue / 5.0) as u8;
        let container_kind = self.known.containers.find("<drop>").unwrap();
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let item = self.inventory.items_id.introduce().one(ItemId);
        let items = vec![Item {
            id: item,
            kind: crop_kind.residue.clone(),
            container,
            quantity,
        }];
        let create_residue = self
            .inventory
            .add_container(container, &container_kind, items)?;

        let events = occur![
            destroy_plant_barrier,
            destroy_plant(),
            self.universe.vanish_crop(crop),
            create_barrier(),
            create_residue(),
            self.appear_stack(container, barrier),
        ];

        Ok(events)
    }
}
