use crate::api::{ActionError, Event};
use crate::assembling::Rotation;
use crate::inventory::FunctionsQuery;
use crate::model::{Activity, AssemblyKey, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn start_assembly(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        pivot: [usize; 2],
        rotation: Rotation,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let key = item.kind.functions.as_assembly(AssemblyKey)?;
        let assembly_kind = self.known.assembly.get(key)?;
        let valid = self.is_placement_valid(farmland, pivot, rotation, &assembly_kind)?;
        let (placement, start_placement) =
            self.assembling.start_placement(rotation, pivot, valid)?;
        let events = occur![
            start_placement(),
            self.appear_assembling_activity(farmer, key, placement),
        ];
        Ok(events)
    }
}
