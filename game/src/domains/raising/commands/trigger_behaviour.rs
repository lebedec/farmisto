use crate::raising::Raising::BehaviourTriggered;
use crate::raising::{AnimalId, Behaviour, Raising, RaisingDomain, RaisingError};

impl RaisingDomain {
    pub fn trigger_behaviour(
        &mut self,
        id: AnimalId,
        trigger: Behaviour,
        behaviour: Behaviour,
    ) -> Result<impl FnOnce() -> Vec<Raising> + '_, RaisingError> {
        let animal = self.get_animal_mut(id)?;
        let command = move || {
            animal.behaviour = behaviour;
            vec![BehaviourTriggered {
                id,
                trigger,
                behaviour,
            }]
        };
        Ok(command)
    }
}
