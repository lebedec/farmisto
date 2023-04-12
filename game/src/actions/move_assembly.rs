use crate::api::{ActionError, Event};
use crate::assembling::{Binding, Part, Rotation};
use crate::collections::Shared;
use crate::math::{Tile, TileMath};
use crate::model::{AssemblyKind, AssemblyTarget, Farmer, Farmland};
use crate::{occur, Game};
use log::info;

impl Game {
    pub(crate) fn move_assembly(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        pivot: Tile,
        rotation: Rotation,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        let assembly = activity.as_assembling()?;
        let assembly_kind = self.known.assembly.get(assembly.key)?;
        let placement = self.assembling.get_placement(assembly.placement)?;
        let valid = self.is_placement_valid(farmland, pivot, rotation, &assembly_kind)?;
        let update_placement =
            self.assembling
                .update_placement(assembly.placement, rotation, pivot, valid)?;
        let events = occur![update_placement(),];
        Ok(events)
    }
}
