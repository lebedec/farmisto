use crate::api::{ActionError, Event};
use crate::assembling::{Binding, Part, Rotation};
use crate::collections::Shared;
use crate::math::{Tile, TileMath};
use crate::model::{AssemblyKind, AssemblyTarget, Farmer, Farmland};
use crate::{occur, Game};

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
        let valid = self.validate_assembly_placement(
            farmland,
            placement.pivot,
            placement.rotation,
            &assembly_kind,
        )?;
        let update_placement =
            self.assembling
                .update_placement(assembly.placement, rotation, pivot, valid)?;
        let events = occur![update_placement(),];
        Ok(events)
    }

    pub(crate) fn validate_assembly_placement(
        &self,
        farmland: Farmland,
        pivot: Tile,
        rotation: Rotation,
        kind: &Shared<AssemblyKind>,
    ) -> Result<bool, ActionError> {
        // TODO: maybe move to asset definition
        let parts: Vec<Part> = match &kind.target {
            AssemblyTarget::Door { .. } => vec![Part {
                binding: Binding::Doorway,
                offset: [0, 0],
            }],
            AssemblyTarget::Cementer { cementer } => vec![
                Part {
                    binding: Binding::Ground,
                    offset: rotation.apply_i8(cementer.input_offset),
                },
                Part {
                    binding: Binding::Ground,
                    offset: [0, 0],
                },
                Part {
                    binding: Binding::Ground,
                    offset: rotation.apply_i8(cementer.output_offset),
                },
            ],
        };

        for part in parts {
            let tile = pivot.add_offset(part.offset);
            let grid = self.building.get_grid(farmland.grid)?;
            let cell = grid.get_cell(tile)?;
            let barrier = self
                .physics
                .get_barrier_at(farmland.space, tile.to_position());
            let valid = match part.binding {
                Binding::Doorway => cell.door,
                Binding::Ground => !cell.wall && barrier.is_none(),
            };
            if !valid {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
