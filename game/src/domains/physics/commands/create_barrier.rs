use crate::collections::Shared;
use crate::math::VectorMath;
use crate::physics::{
    Barrier, BarrierId, BarrierKind, Physics, PhysicsDomain, PhysicsError, SpaceId,
};

impl PhysicsDomain {
    pub fn create_barrier(
        &mut self,
        space: SpaceId,
        kind: Shared<BarrierKind>,
        position: [f32; 2],
        active: bool,
        overlapping: bool,
    ) -> Result<(BarrierId, impl FnOnce() -> Vec<Physics> + '_), PhysicsError> {
        let id = BarrierId(self.barriers_sequence + 1);
        let barrier = Barrier {
            id,
            kind,
            position,
            space,
            active,
        };
        let i: usize = space.into();
        if !overlapping {
            for barrier in &self.barriers[i] {
                if barrier.position.to_tile() == position.to_tile() {
                    return Err(PhysicsError::BarrierCreationOverlaps { other: barrier.id });
                }
            }
        }
        let operation = move || {
            let events = vec![Physics::BarrierCreated {
                id: barrier.id,
                key: barrier.kind.id,
                space: barrier.space,
                position: barrier.position,
                active: barrier.active,
            }];
            self.barriers_sequence += 1;
            self.barriers[i].push(barrier);
            events
        };
        Ok((id, operation))
    }
}
