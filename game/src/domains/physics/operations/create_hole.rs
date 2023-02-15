use crate::physics::Physics::SpaceUpdated;
use crate::physics::{Physics, PhysicsDomain, PhysicsError, SpaceId};

impl PhysicsDomain {
    pub fn create_hole<'operation>(
        &'operation mut self,
        id: SpaceId,
        hole: [usize; 2],
    ) -> Result<impl FnOnce() -> Vec<Physics> + 'operation, PhysicsError> {
        let space = self.get_space_mut(id)?;
        let [hole_x, hole_y] = hole;

        if space.holes[hole_y][hole_x] == 1 {
            return Err(PhysicsError::HoleAlreadyExists { hole });
        }

        let operation = move || {
            space.holes[hole_y][hole_x] = 1;
            vec![SpaceUpdated {
                id,
                holes: space.holes.clone(),
            }]
        };
        Ok(operation)
    }
}
