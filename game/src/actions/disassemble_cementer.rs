use crate::api::{ActionError, Event};
use crate::inventory::FunctionsQuery;
use crate::model::{Activity, AssemblyKey, Cementer, Farmer};
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
        let (_item, create_kit) =
            self.inventory
                .create_item(&cementer_kind.kit, farmer.hands, 1)?;

        // TODO: destroy input + output + items

        let events = occur![
            destroy_barrier(),
            create_kit(),
            self.universe.vanish_cementer(cementer),
            self.appear_assembling_activity(farmer, key, placement),
        ];

        Ok(events)
    }
}
