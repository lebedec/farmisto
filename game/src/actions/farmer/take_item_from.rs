use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::math::TileMath;
use crate::model::{Activity, Cementer, Composter, Construction, Farmer, Farmland, Stack};
use crate::{occur, Game};

impl Game {
    pub(crate) fn take_item_from_stack(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        stack: Stack,
    ) -> Result<Vec<Event>, ActionError> {
        let destination = self.physics.get_barrier(stack.barrier)?.position;
        self.ensure_target_reachable(farmer.body, destination)?;
        self.take_item_from_container(farmer, stack.container)
    }

    pub(crate) fn take_item_from_construction(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let surveyor = self.building.get_surveyor(construction.surveyor)?;
        let stake = surveyor.get_stake(construction.stake)?;
        let destination = stake.cell.position();
        self.ensure_target_reachable(farmer.body, destination)?;
        self.take_item_from_container(farmer, construction.container)
    }

    pub(crate) fn take_item_from_cementer(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        cementer: Cementer,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let placement = self.assembling.get_placement(cementer.placement)?;
        let cementer_kind = self.known.cementers.get(cementer.key)?;
        if container == cementer.input {
            let offset = placement.rotation.apply_i8(cementer_kind.input_offset);
            let destination = placement.pivot.add_offset(offset).position();
            self.ensure_target_reachable(farmer.body, destination)?;
            self.take_item_from_container(farmer, cementer.input)
        } else {
            let offset = placement.rotation.apply_i8(cementer_kind.output_offset);
            let destination = placement.pivot.add_offset(offset).position();
            self.ensure_target_reachable(farmer.body, destination)?;
            self.take_item_from_container(farmer, cementer.output)
        }
    }

    pub(crate) fn take_item_from_composter(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        composter: Composter,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let placement = self.assembling.get_placement(composter.placement)?;
        let composter_kind = self.known.composters.get(composter.key)?;
        if container == composter.input {
            let offset = placement.rotation.apply_i8(composter_kind.input_offset);
            let destination = placement.pivot.add_offset(offset).position();
            self.ensure_target_reachable(farmer.body, destination)?;
            self.take_item_from_container(farmer, composter.input)
        } else {
            let offset = placement.rotation.apply_i8(composter_kind.output_offset);
            let destination = placement.pivot.add_offset(offset).position();
            self.ensure_target_reachable(farmer.body, destination)?;
            self.take_item_from_container(farmer, composter.output)
        }
    }

    pub(crate) fn take_item_from_container(
        &mut self,
        farmer: Farmer,
        container: ContainerId,
    ) -> Result<Vec<Event>, ActionError> {
        let pop_item = self.inventory.pop_item(container, farmer.hands)?;
        // TODO: quantity merge
        let events = occur![
            pop_item(),
            self.universe.change_activity(farmer, Activity::Usage),
        ];
        Ok(events)
    }
}
