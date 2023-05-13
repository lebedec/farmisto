use crate::physics::{BodyId, Physics, PhysicsDomain, PhysicsError};

impl PhysicsDomain {
    pub fn stop_body(
        &mut self,
        id: BodyId,
    ) -> Result<impl FnOnce() -> Vec<Physics> + '_, PhysicsError> {
        let body = self.get_body_mut(id)?;
        let command = move || {
            body.destination = body.position;
            vec![Physics::BodyPositionChanged {
                id: body.id.into(),
                space: body.space,
                position: body.position,
                destination: body.destination,
            }]
        };
        Ok(command)
    }
}
