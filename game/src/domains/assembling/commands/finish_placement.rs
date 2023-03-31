use crate::assembling::{
    Assembling, AssemblingDomain, AssemblingError, Placement, PlacementId, Rotation,
};

impl AssemblingDomain {
    pub fn finish_placement<'command>(
        &'command mut self,
        id: PlacementId,
    ) -> Result<(Placement, impl FnOnce() -> Vec<Assembling> + 'command), AssemblingError> {
        let placement = self.get_placement(id)?;
        let placement = placement.clone();
        let command = move || {
            // self.placements.remove(&id);
            vec![]
        };
        Ok((placement, command))
    }
}
