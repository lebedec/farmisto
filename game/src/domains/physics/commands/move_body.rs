use crate::physics::{BodyId, Physics, PhysicsDomain, PhysicsError};

impl PhysicsDomain {
    pub fn move_body(
        &mut self,
        id: BodyId,
        destination: [f32; 2],
    ) -> Result<impl FnOnce() -> Vec<Physics> + '_, PhysicsError> {
        let body = self.get_body_mut(id)?;
        let command = move || {
            body.destination = destination;
            vec![]
        };
        Ok(command)
    }
}
