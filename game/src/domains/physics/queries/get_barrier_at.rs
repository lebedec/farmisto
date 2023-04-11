use crate::math::test_rect_collision;
use crate::physics::{Barrier, PhysicsDomain, PhysicsError, SpaceId};

impl PhysicsDomain {
    pub fn get_barrier_at(&self, space: SpaceId, position: [f32; 2]) -> Option<&Barrier> {
        for barrier in self.barriers[space.0].iter() {
            if test_rect_collision(position, [0.1; 2], barrier.position, barrier.kind.bounds) {
                return Some(barrier);
            }
        }
        return None;
    }
}
