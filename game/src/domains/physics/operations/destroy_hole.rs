use crate::physics::Physics::SpaceUpdated;
use crate::physics::{Physics, PhysicsDomain, PhysicsError, SpaceId};

impl PhysicsDomain {
    pub fn destroy_hole<'operation>(
        &'operation mut self,
        id: SpaceId,
        hole: [usize; 2],
    ) -> Result<impl FnOnce() -> Vec<Physics> + 'operation, PhysicsError> {
        let space = self.get_space_mut(id)?;
        let [hole_x, hole_y] = hole;

        // if space.holes[hole_y][hole_x] == 0 {
        //     return Err(PhysicsError::HoleNotFound { hole });
        // }

        let operation = move || {
            space.holes[hole_y][hole_x] = 0;
            vec![SpaceUpdated {
                id,
                holes: space.holes.clone(),
            }]
        };
        Ok(operation)
    }
}
