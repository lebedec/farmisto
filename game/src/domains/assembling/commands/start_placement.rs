use crate::assembling::{
    Assembling, AssemblingDomain, AssemblingError, Placement, PlacementId, Rotation,
};

impl AssemblingDomain {
    pub fn start_placement<'command>(
        &'command mut self,
        rotation: Rotation,
        pivot: [usize; 2],
    ) -> Result<(PlacementId, impl FnOnce() -> Vec<Assembling> + 'command), AssemblingError> {
        let id = PlacementId(self.placements_id + 1);
        let placement = Placement {
            id,
            rotation,
            pivot,
        };
        let command = move || {
            self.placements_id += 1;
            self.placements.insert(id, placement);
            vec![]
        };
        Ok((id, command))
    }
}
