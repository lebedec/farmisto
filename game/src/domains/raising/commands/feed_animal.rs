use crate::raising::Raising::{AnimalChanged, BehaviourTriggered};
use crate::raising::{AnimalId, Behaviour, Raising, RaisingDomain, RaisingError};

impl RaisingDomain {
    pub fn feed_animal(
        &mut self,
        id: AnimalId,
        food: f32,
    ) -> Result<impl FnOnce() -> Vec<Raising> + '_, RaisingError> {
        let animal = self.get_animal_mut(id)?;
        let command = move || {
            animal.hunger = (animal.hunger - food).max(0.0);
            animal.behaviour = Behaviour::Idle;
            vec![
                AnimalChanged {
                    id,
                    hunger: animal.hunger,
                    thirst: animal.thirst,
                },
                BehaviourTriggered {
                    id,
                    trigger: Behaviour::Eating,
                    behaviour: Behaviour::Idle,
                },
            ]
        };
        Ok(command)
    }
}
