use crate::assembling::{AssemblingDomain, AssemblingError, Placement, PlacementId};

impl AssemblingDomain {
    pub fn get_placement(&self, id: PlacementId) -> Result<&Placement, AssemblingError> {
        self.placements
            .get(&id)
            .ok_or(AssemblingError::PlacementNotFound { id })
    }

    pub fn get_placement_mut(
        &mut self,
        id: PlacementId,
    ) -> Result<&mut Placement, AssemblingError> {
        self.placements
            .get_mut(&id)
            .ok_or(AssemblingError::PlacementNotFound { id })
    }
}
