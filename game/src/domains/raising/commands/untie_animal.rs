use crate::raising::Raising::AnimalUntied;
use crate::raising::{Raising, RaisingDomain, RaisingError, TetherId};

impl RaisingDomain {
    pub fn untie_animal(
        &mut self,
        tether: TetherId,
    ) -> Result<impl FnOnce() -> Vec<Raising> + '_, RaisingError> {
        let tether = self.get_tether_mut(tether)?;
        let command = move || {
            let mut events = vec![];
            if let Some(other) = tether.animal {
                events.push(AnimalUntied {
                    id: other,
                    tether: tether.id,
                })
            };
            tether.animal = None;
            events
        };
        Ok(command)
    }
}
