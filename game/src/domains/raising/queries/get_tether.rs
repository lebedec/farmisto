use crate::raising::{RaisingDomain, RaisingError, Tether, TetherId};

impl RaisingDomain {
    pub fn get_tether_mut(&mut self, id: TetherId) -> Result<&mut Tether, RaisingError> {
        self.tethers
            .iter_mut()
            .find(|tether| tether.id == id)
            .ok_or(RaisingError::TetherNotFound { id })
    }
}
