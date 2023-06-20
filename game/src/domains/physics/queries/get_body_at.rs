use crate::math::VectorMath;
use crate::physics::{Body, PhysicsDomain, PhysicsError, SpaceId};

impl PhysicsDomain {
    pub fn get_body_at(&self, space: SpaceId, position: [f32; 2]) -> Result<&Body, PhysicsError> {
        for body in self.bodies[space.0].iter() {
            if body.position.to_tile() == position.to_tile() {
                return Ok(body);
            }
        }
        return Err(PhysicsError::BodyNotFoundAt { position });
    }
}
