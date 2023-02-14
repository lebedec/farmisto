use crate::collections::Shared;
use crate::math::{move_with_collisions, VectorMath};
use crate::physics::{
    Barrier, BarrierId, BarrierKind, Physics, PhysicsDomain, PhysicsError, SpaceId,
};

impl PhysicsDomain {
    pub fn create_barrier<'operation>(
        &'operation mut self,
        space: SpaceId,
        kind: Shared<BarrierKind>,
        position: [f32; 2],
        overlapping: bool,
    ) -> Result<(BarrierId, impl FnOnce() -> Vec<Physics> + 'operation), PhysicsError> {
        let id = BarrierId(self.barriers_sequence + 1);
        let barrier = Barrier {
            id,
            kind,
            position,
            space,
        };
        if !overlapping {
            for barrier in &self.barriers[space.0] {
                if barrier.position.to_tile() == position.to_tile() {
                    return Err(PhysicsError::BarrierCreationOverlaps { other: barrier.id });
                }
            }
        }
        let operation = move || {
            let events = vec![Physics::BarrierCreated {
                id: barrier.id,
                space: barrier.space,
                position: barrier.position,
                bounds: barrier.kind.bounds,
            }];
            self.barriers_sequence += 1;
            self.barriers[space.0].push(barrier);
            events
        };
        Ok((id, operation))
    }
}
