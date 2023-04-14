use crate::landscaping::{Land, LandscapingDomain};

impl LandscapingDomain {
    pub fn load_lands(&mut self, lands: Vec<Land>) {
        for land in lands {
            self.lands_id.register(land.id.0);
            self.lands.insert(land.id, land);
        }
    }
}
