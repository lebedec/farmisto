use crate::physics::Physics::BarrierDestroyed;
use crate::physics::{BarrierId, Physics, PhysicsDomain, PhysicsError};

impl PhysicsDomain {
    pub fn destroy_barrier(
        & mut self,
        id: BarrierId,
    ) -> Result<impl FnOnce() -> Vec<Physics> + '_, PhysicsError> {
        let barrier = self.get_barrier(id)?;
        let space = barrier.space;
        let command = move || {
            let index = self.barriers[space.0]
                .iter()
                .position(|barrier| barrier.id == id)
                .unwrap();
            let barrier = self.barriers[space.0].remove(index);

            vec![BarrierDestroyed {
                id,
                space,
                position: barrier.position,
            }]
        };
        Ok(command)
    }
}
