use crate::landscaping::{Land, LandId, LandKind, Landscaping, LandscapingDomain, LandscapingError, Place, Surface};
use crate::math::{ArrayIndex};
use crate::timing::Shared;

impl LandscapingDomain {
    pub fn create_land(
        &mut self,
        kind: &Shared<LandKind>,
    ) -> Result<(LandId, impl FnOnce() -> Vec<Landscaping> + '_), LandscapingError> {
        let id = self.lands_id.introduce().one(LandId);
        let land = Land {
            id,
            kind: kind.clone(),
            moisture: vec![0.0; 128 * 128],
            moisture_capacity: vec![0.0; 128 * 128],
            surface: vec![0; 128 * 128],
        };
        let command = move || {
            self.lands_id.register(id.0);
            self.lands.insert(id, land);
            vec![]
        };
        Ok((id, command))
    }
}
