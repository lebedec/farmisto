use crate::raising::Raising::AnimalChanged;
use crate::raising::{AnimalId, Raising, RaisingDomain, RaisingError};

impl RaisingDomain {
    pub fn feed_animal(
        &mut self,
        id: AnimalId,
        food: f32,
    ) -> Result<impl FnOnce() -> Vec<Raising> + '_, RaisingError> {
        let animal = self.get_animal_mut(id)?;
        let command = move || {
            animal.hunger = (animal.hunger - food).max(0.0);
            vec![AnimalChanged {
                id,
                hunger: animal.hunger,
                thirst: animal.thirst,
            }]
        };
        Ok(command)
    }
}
