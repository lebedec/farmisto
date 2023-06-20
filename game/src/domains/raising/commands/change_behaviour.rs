use crate::raising::Raising::BehaviourChanged;
use crate::raising::{AnimalId, Behaviour, Raising, RaisingDomain, RaisingError};

impl RaisingDomain {
    pub fn change_behaviour(
        &mut self,
        id: AnimalId,
        behaviour: Behaviour,
    ) -> Result<impl FnOnce() -> Vec<Raising> + '_, RaisingError> {
        let animal = self.get_animal_mut(id)?;
        let command = move || {
            animal.behaviour = behaviour;
            vec![BehaviourChanged { id, behaviour }]
        };
        Ok(command)
    }
}
