use crate::api::{ActionError, Event};
use crate::inventory::{FunctionsQuery, ItemId};
use crate::model::{Activity, AssemblyKey, Door, Farmer};
use crate::{occur, Game};

impl Game {
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
