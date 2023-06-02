use crate::landscaping::{LandId, Landscaping, LandscapingDomain, LandscapingError};
use crate::math::Rect;

impl LandscapingDomain {
    pub fn inspect_land(&self, _land: LandId, _rect: Rect) -> Result<Landscaping, LandscapingError> {
        unimplemented!()
    }
}
