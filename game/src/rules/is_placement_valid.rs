use crate::api::ActionError;
use crate::assembling::{Binding, Part, Rotation};
use crate::collections::Shared;
use crate::math::{Tile, TileMath};
use crate::model::{AssemblyKind, AssemblyTarget, Farmland};
use crate::Game;

impl Game {
    pub(crate) fn is_placement_valid(
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
            AssemblyTarget::Composter { composter } => vec![
                Part {
                    binding: Binding::Ground,
                    offset: rotation.apply_i8(composter.input_offset),
                },
                Part {
                    binding: Binding::Ground,
                    offset: [0, 0],
                },
                Part {
                    binding: Binding::Ground,
                    offset: rotation.apply_i8(composter.output_offset),
                },
            ],
            AssemblyTarget::Rest { .. } => vec![Part {
                binding: Binding::Ground,
                offset: [0, 0],
            }],
        };

        for part in parts {
            let tile = pivot.add_offset(part.offset);
            let grid = self.building.get_grid(farmland.grid)?;
            let cell = grid.get_cell(tile)?;
            let barrier = self.physics.get_barrier_at(farmland.space, tile.position());
            let valid = match part.binding {
                Binding::Doorway => cell.door,
                Binding::Ground => !cell.wall && barrier.is_none(),
            };
            return Ok(valid);
        }

        Ok(true)
    }
}
