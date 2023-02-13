use crate::collections::Shared;
use crate::math::move_with_collisions;
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
            let barriers = &self.barriers[space.0];
            let destination = move_with_collisions(&barrier, position, barriers);
            if destination.is_none() {
                return Err(PhysicsError::BarrierCreationOverlaps);
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
