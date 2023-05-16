use crate::raising::Raising::{AnimalTied, AnimalUntied};
use crate::raising::{AnimalId, Raising, RaisingDomain, RaisingError, TetherId};

impl RaisingDomain {
    pub fn tie_animal(
        &mut self,
        tether: TetherId,
        animal: AnimalId,
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
            tether.animal = Some(animal);
            events.push(AnimalTied {
                id: animal,
                tether: tether.id,
            });
            events
        };
        Ok(command)
    }
}
