use crate::landscaping::{LandId, Landscaping, LandscapingDomain, LandscapingError};
use crate::math::Rect;

impl LandscapingDomain {
    pub fn inspect_land(&self, land: LandId, rect: Rect) -> Result<Landscaping, LandscapingError> {
        unimplemented!()
    }
}
