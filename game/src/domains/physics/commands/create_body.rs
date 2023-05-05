use crate::collections::{Shared, TemporaryRef};
use crate::physics::{
    BarrierId, BarrierKind, Body, BodyId, BodyKind, Physics, PhysicsDomain, PhysicsError, SpaceId,
};

impl PhysicsDomain {
    pub fn create_body(
        &mut self,
        id: BodyId,
        space: SpaceId,
        kind: Shared<BodyKind>,
        position: [f32; 2],
    ) -> Result<impl FnOnce() -> Vec<Physics>, PhysicsError> {
        let body = Body {
            id,
            kind: kind.clone(),
            position,
            destination: position,
            space,
        };
        let mut domain = TemporaryRef::from(self);
        let command = move || {
            let mut events = vec![];
            domain.bodies_sequence.register(id.0);
            domain.bodies[space.0].push(body);
            events
        };
        Ok(command)
    }
}