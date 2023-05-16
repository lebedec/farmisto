use crate::raising::Raising::AnimalUntied;
use crate::raising::RaisingError::TetherNotFound;
use crate::raising::{Raising, RaisingDomain, RaisingError, TetherId};

impl RaisingDomain {
    pub fn destroy_tether(
        &mut self,
        id: TetherId,
    ) -> Result<impl FnOnce() -> Vec<Raising> + '_, RaisingError> {
        let index = self
            .tethers
            .iter()
            .position(|tether| tether.id == id)
            .ok_or(TetherNotFound { id })?;
        let command = move || {
            let mut events = vec![];
            let tether = self.tethers.remove(index);
            if let Some(animal) = tether.animal {
                events.push(AnimalUntied {
                    id: animal,
                    tether: tether.id,
                });
            }
            events
        };
        Ok(command)
    }
}
