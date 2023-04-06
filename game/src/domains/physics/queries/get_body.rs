use crate::physics::{Body, BodyId, PhysicsDomain, PhysicsError};

impl PhysicsDomain {
    pub fn get_body_mut(&mut self, id: BodyId) -> Result<&mut Body, PhysicsError> {
        for bodies in self.bodies.iter_mut() {
            for body in bodies {
                if body.id == id {
                    return Ok(body);
                }
            }
        }
        return Err(PhysicsError::BodyNotFound { id });
    }

    pub fn get_body(&self, id: BodyId) -> Result<&Body, PhysicsError> {
        for bodies in self.bodies.iter() {
            for body in bodies {
                if body.id == id {
                    return Ok(body);
                }
            }
        }
        return Err(PhysicsError::BodyNotFound { id });
    }
}
