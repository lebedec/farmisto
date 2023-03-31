use crate::raising::Raising::AnimalChanged;
use crate::raising::{AnimalId, Raising, RaisingDomain, RaisingError};

impl RaisingDomain {
    pub fn feed_animal<'operation>(
        &'operation mut self,
        id: AnimalId,
        food: f32,
    ) -> Result<impl FnOnce() -> Vec<Raising> + 'operation, RaisingError> {
        let animal = self.get_animal_mut(id)?;
        let operation = move || {
            animal.hunger = (animal.hunger - food).max(0.0);
            vec![AnimalChanged {
                id,
                hunger: animal.hunger,
            }]
        };
        Ok(operation)
    }
}
