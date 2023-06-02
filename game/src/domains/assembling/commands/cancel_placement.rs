use crate::assembling::{Assembling, AssemblingDomain, AssemblingError, PlacementId};

impl AssemblingDomain {
    pub fn cancel_placement(
        &mut self,
        id: PlacementId,
    ) -> Result<impl FnOnce() -> Vec<Assembling> + '_, AssemblingError> {
        self.get_placement(id)?;
        let command = move || {
            self.placements.remove(&id);
            vec![]
        };
        Ok(command)
    }
}
