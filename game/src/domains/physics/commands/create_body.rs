use crate::collections::{Shared, TrustedRef};
use crate::physics::{
    Body, BodyId, BodyKind, Physics, PhysicsDomain, PhysicsError, SpaceId,
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
        let mut domain = TrustedRef::from(self);
        let command = move || {
            let events = vec![];
            domain.bodies_sequence.register(id.0);
            domain.bodies[space.0].push(body);
            events
        };
        Ok(command)
    }
}
