use crate::assembling::{Assembling, AssemblingDomain, AssemblingError, Placement, PlacementId};

impl AssemblingDomain {
    pub fn finish_placement(
        &mut self,
        id: PlacementId,
    ) -> Result<(Placement, impl FnOnce() -> Vec<Assembling> + '_), AssemblingError> {
        let placement = self.get_placement(id)?;
        let placement = placement.clone();
        let command = move || {
            // self.placements.remove(&id);
            vec![]
        };
        Ok((placement, command))
    }
}
