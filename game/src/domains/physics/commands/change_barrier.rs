use crate::physics::{BarrierId, Physics, PhysicsDomain, PhysicsError};

impl PhysicsDomain {
    pub fn change_barrier(&mut self, id: BarrierId, active: bool) -> Result<impl FnOnce() -> Vec<Physics> + '_, PhysicsError> {
        let barrier = self.get_barrier_mut(id)?;
        let command = move || {
            barrier.active = active;
            vec![Physics::BarrierChanged {
                id,
                space: barrier.space,
                active: barrier.active,
            }]
        };
        Ok(command)
    }
}
