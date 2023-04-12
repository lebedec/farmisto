use crate::physics::{PhysicsDomain, SpaceId};

impl PhysicsDomain {
    pub fn cast_ray(&self, space: SpaceId, from: [f32; 2], to: [f32; 2]) -> Vec<[f32; 2]> {
        vec![]
    }
}
