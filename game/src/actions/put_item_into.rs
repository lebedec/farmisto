use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::math::TileMath;
use crate::model::{Activity, Cementer, Construction, Farmer, Farmland, Stack};
use crate::{occur, Game};

impl Game {
    pub(crate) fn put_item_into_stack(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        stack: Stack,
    ) -> Result<Vec<Event>, ActionError> {
        let destination = self.physics.get_barrier(stack.barrier)?.position;
        self.ensure_target_reachable(farmland.space, farmer, destination)?;
        self.put_item_into_container(farmer, stack.container)
    }

    pub(crate) fn put_item_into_construction(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let destination = construction.cell.to_position();
        self.ensure_target_reachable(farmland.space, farmer, destination)?;
        self.put_item_into_container(farmer, construction.container)
    }

    pub(crate) fn put_item_into_cementer(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        cementer: Cementer,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let placement = self.assembling.get_placement(cementer.placement)?;
        let cementer_kind = self.known.cementers.get(cementer.key)?;
        if container == cementer.input {
            let offset = placement.rotation.apply_i8(cementer_kind.input_offset);
            let destination = placement.pivot.add_offset(offset).to_position();
            self.ensure_target_reachable(farmland.space, farmer, destination)?;
            self.put_item_into_container(farmer, cementer.input)
        } else {
            let offset = placement.rotation.apply_i8(cementer_kind.output_offset);
            let destination = placement.pivot.add_offset(offset).to_position();
            self.ensure_target_reachable(farmland.space, farmer, destination)?;
            self.put_item_into_container(farmer, cementer.output)
        }
    }

    pub(crate) fn put_item_into_container(
        &mut self,
        farmer: Farmer,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let hands = self.inventory.get_container(farmer.hands)?;
        let is_last_item = hands.items.len() <= 1;
        let transfer = self.inventory.pop_item(farmer.hands, container)?;
        // TODO: quantity merge
        let activity = if is_last_item {
            self.universe.change_activity(farmer, Activity::Idle)
        } else {
            vec![]
        };
        let events = occur![transfer(), activity,];
        Ok(events)
    }
}