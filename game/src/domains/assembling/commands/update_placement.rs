use crate::assembling::Assembling::PlacementUpdated;
use crate::assembling::{Assembling, AssemblingDomain, AssemblingError, PlacementId, Rotation};

impl AssemblingDomain {
    pub fn update_placement(
        &mut self,
        id: PlacementId,
        rotation: Rotation,
        pivot: [usize; 2],
    ) -> Result<impl FnOnce() -> Vec<Assembling> + '_, AssemblingError> {
        let placement = self.get_placement_mut(id)?;
        let command = move || {
            placement.rotation = rotation;
            placement.pivot = pivot;
            vec![PlacementUpdated {
                placement: placement.id,
                rotation: placement.rotation,
                pivot: placement.pivot,
            }]
        };
        Ok(command)
    }
}
