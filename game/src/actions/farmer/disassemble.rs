use crate::api::{ActionError, Event};
use crate::inventory::{FunctionsQuery, ItemId};
use crate::model::{Activity, AssemblyKey, Cementer, Composter, Door, Farmer, Rest};
use crate::{occur, Game};

impl Game {
    pub fn disassemble_cementer(
        &mut self,
        farmer: Farmer,
        cementer: Cementer,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        let cementer_kind = self.known.cementers.get(cementer.key)?;
        let key = cementer_kind.kit.functions.as_assembly(AssemblyKey)?;
        let placement = cementer.placement;

        let destroy_barrier = self.physics.destroy_barrier(cementer.barrier)?;
        let item = self.inventory.items_id.introduce().one(ItemId);
        let create_kit = self
            .inventory
            .create_item(item, &cementer_kind.kit, farmer.hands, 1)?;

        // TODO: destroy input + output + items

        let events = occur![
            destroy_barrier(),
            create_kit(),
            self.universe.vanish_cementer(cementer),
            self.appear_assembling_activity(farmer, key, placement),
        ];

        Ok(events)
    }

    pub fn disassemble_composter(
        &mut self,
        farmer: Farmer,
        composter: Composter,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        let composter_kind = self.known.composters.get(composter.key)?;
        let key = composter_kind.kit.functions.as_assembly(AssemblyKey)?;
        let placement = composter.placement;

        let destroy_barrier = self.physics.destroy_barrier(composter.barrier)?;
        let item = self.inventory.items_id.introduce().one(ItemId);
        let create_kit = self
            .inventory
            .create_item(item, &composter_kind.kit, farmer.hands, 1)?;

        // TODO: destroy input + output + items

        let events = occur![
            destroy_barrier(),
            create_kit(),
            self.universe.vanish_composter(composter),
            self.appear_assembling_activity(farmer, key, placement),
        ];

        Ok(events)
    }

    pub fn disassemble_rest(
        &mut self,
        farmer: Farmer,
        rest: Rest,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        let rest_kind = self.known.rests.get(rest.key)?;
        let key = rest_kind.kit.functions.as_assembly(AssemblyKey)?;
        let placement = rest.placement;

        let destroy_barrier = self.physics.destroy_barrier(rest.barrier)?;
        let item = self.inventory.items_id.introduce().one(ItemId);
        let create_kit = self
            .inventory
            .create_item(item, &rest_kind.kit, farmer.hands, 1)?;

        let events = occur![
            destroy_barrier(),
            create_kit(),
            self.universe.vanish_rest(rest),
            self.appear_assembling_activity(farmer, key, placement),
        ];

        Ok(events)
    }

    pub fn disassemble_door(
        &mut self,
        farmer: Farmer,
        door: Door,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        let door_kind = self.known.doors.get(door.key)?;
        let key = door_kind.kit.functions.as_assembly(AssemblyKey)?;
        let placement = door.placement;

        let destroy_barrier = self.physics.destroy_barrier(door.barrier)?;
        let item = self.inventory.items_id.introduce().one(ItemId);
        let create_kit = self
            .inventory
            .create_item(item, &door_kind.kit, farmer.hands, 1)?;

        let events = occur![
            destroy_barrier(),
            create_kit(),
            self.universe.vanish_door(door),
            self.appear_assembling_activity(farmer, key, placement),
        ];

        Ok(events)
    }
}
