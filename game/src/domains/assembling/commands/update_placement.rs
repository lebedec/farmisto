use crate::assembling::{
    Assembling, AssemblingDomain, AssemblingError, Placement, PlacementId, Rotation,
};
use crate::assembling::Assembling::PlacementUpdated;

impl AssemblingDomain {
    pub fn update_placement<'command>(
        &'command mut self,
        id: PlacementId,
        rotation: Rotation,
        pivot: [usize; 2],
    ) -> Result<impl FnOnce() -> Vec<Assembling> + 'command, AssemblingError> {
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
