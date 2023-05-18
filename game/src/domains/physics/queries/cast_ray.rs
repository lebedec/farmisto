use crate::math::{cast_ray, VectorMath};
use crate::physics::{PhysicsDomain, PhysicsError, SpaceId};

impl PhysicsDomain {
    pub fn cast_ray(
        &self,
        space: SpaceId,
        start: [f32; 2],
        end: [f32; 2],
        filter: &[u8],
    ) -> Result<Vec<[f32; 2]>, PhysicsError> {
        let space = self.get_space(space)?;
        let mut holes = space.holes.clone();
        for barrier in self.barriers[space.id.0].iter() {
            let [x, y] = barrier.position.to_tile();
            // TODO: barrier size
            holes[y][x] = 1;
        }
        let contacts = cast_ray(start, end, &holes, filter);
        Ok(contacts)
    }
}
