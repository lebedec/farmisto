use crate::api::{ActionError, Event};
use crate::inventory::{FunctionsQuery, InventoryError, ItemId};
use crate::model::{Activity, Crop, CropKey, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn harvest_crop(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        crop: Crop,
    ) -> Result<Vec<Event>, ActionError> {
        let destination = self.physics.get_barrier(crop.barrier)?.position;
        self.ensure_target_reachable(farmer.body, destination)?;
        let crop_kind = self.known.crops.get(crop.key).unwrap();
        let item_kind = &crop_kind.fruits;
        let (new_harvest, capacity) = match self.inventory.get_container_item(farmer.hands) {
            Ok(item) => {
                let kind = item.kind.functions.as_product()?;
                if crop.key != CropKey(kind) {
                    return Err(InventoryError::ItemFunctionNotFound.into());
                }
                (false, item.kind.max_quantity - item.quantity)
            }
            _ => (true, item_kind.max_quantity),
        };
        let (fruits, harvest) = self.planting.harvest_plant(crop.plant, capacity)?;
        let events = if new_harvest {
            let item = self.inventory.items_id.introduce().one(ItemId);
            let create_item = self
                .inventory
                .create_item(item, item_kind, farmer.hands, fruits)?;
            let change_activity = self.universe.change_activity(farmer, Activity::Usage);
            occur![harvest(), create_item(), change_activity,]
        } else {
            let increase_item = self.inventory.increase_item(farmer.hands, fruits)?;

            occur![harvest(), increase_item(),]
        };
        Ok(events)
    }
}
