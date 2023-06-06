use crate::collections::Shared;
use crate::physics::{
    BarrierId, BarrierKind, Physics, PhysicsDomain, PhysicsError, Space, SpaceId, SpaceKind,
};

impl PhysicsDomain {
    pub fn create_space(
        &mut self,
        kind: &Shared<SpaceKind>,
    ) -> Result<(SpaceId, impl FnOnce() -> Vec<Physics> + '_), PhysicsError> {
        let id = SpaceId(self.spaces_sequence + 1);
        let space = Space {
            id,
            kind: kind.clone(),
            holes: vec![vec![0; 128]; 128],
        };
        let command = move || {
            let events = vec![];
            self.barriers_sequence += 1;
            self.spaces_sequence += 1;
            self.spaces.push(space);
            events
        };
        Ok((id, command))
    }
}
