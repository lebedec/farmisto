use crate::landscaping::{LandId, Landscaping, LandscapingDomain, LandscapingError};
use crate::math::Rect;

impl LandscapingDomain {
    pub fn inspect_land(&self, land: LandId, rect: Rect) -> Result<Landscaping, LandscapingError> {
        let land = self.get_land(land)?;
        let [x, y, w, h] = rect;
        let mut moisture = Vec::with_capacity(w * h);

        moisture[..4].copy_from_slice(&land.moisture[0][1..5]);
        unimplemented!()
    }
}
