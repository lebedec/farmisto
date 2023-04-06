use crate::physics::{Barrier, BarrierId, PhysicsDomain, PhysicsError};

impl PhysicsDomain {

    pub fn get_barrier(&self, id: BarrierId) -> Result<&Barrier, PhysicsError> {
        for barriers in self.barriers.iter() {
            for barrier in barriers {
                if barrier.id == id {
                    return Ok(barrier);
                }
            }
        }
        return Err(PhysicsError::BarrierNotFound { id });
    }

    pub fn get_barrier_mut(&mut self, id: BarrierId) -> Result<&mut Barrier, PhysicsError> {
        for barriers in self.barriers.iter_mut() {
            for barrier in barriers {
                if barrier.id == id {
                    return Ok(barrier);
                }
            }
        }
        return Err(PhysicsError::BarrierNotFound { id });
    }
}