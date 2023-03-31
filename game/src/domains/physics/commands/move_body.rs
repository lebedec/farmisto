use crate::physics::{BodyId, PhysicsDomain, PhysicsError};

impl PhysicsDomain {
    pub fn move_body2(&mut self, id: BodyId, direction: [f32; 2]) -> Result<(), PhysicsError> {
        let body = self.get_body_mut(id)?;
        body.destination = direction;
        Ok(())
    }
}
