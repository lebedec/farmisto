use crate::physics::{PhysicsDomain, PhysicsError, Space, SpaceId};

impl PhysicsDomain {
    pub fn get_space(&self, id: SpaceId) -> Result<&Space, PhysicsError> {
        self.spaces
            .iter()
            .find(|space| space.id == id)
            .ok_or(PhysicsError::SpaceNotFound { space: id })
    }

    pub fn get_space_mut(&mut self, id: SpaceId) -> Result<&mut Space, PhysicsError> {
        self.spaces
            .iter_mut()
            .find(|space| space.id == id)
            .ok_or(PhysicsError::SpaceNotFound { space: id })
    }
}
