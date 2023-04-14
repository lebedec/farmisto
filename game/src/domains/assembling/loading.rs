use crate::assembling::{AssemblingDomain, Placement};

impl AssemblingDomain {
    pub fn load_placements(&mut self, placements: Vec<Placement>, sequence: usize) {
        self.placements_id = sequence;
        for placement in placements {
            self.placements.insert(placement.id, placement);
        }
    }
}
