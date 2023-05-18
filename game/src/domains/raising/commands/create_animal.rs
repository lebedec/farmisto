use crate::raising::{
    Animal, AnimalId, AnimalKind, Behaviour, Raising, RaisingDomain, RaisingError, Tether, TetherId,
};
use crate::timing::Shared;

impl RaisingDomain {
    pub fn create_animal(
        &mut self,
        kind: Shared<AnimalKind>,
    ) -> Result<(AnimalId, impl FnOnce() -> Vec<Raising> + '_), RaisingError> {
        let id = AnimalId(self.animals_id + 1);
        let command = move || {
            self.animals.push(Animal {
                id,
                kind,
                age: 0.0,
                weight: 0.0,
                thirst: 0.0,
                hunger: 0.0,
                voracity: 0.0,
                health: 1.0,
                stress: 0.0,
                behaviour: Behaviour::Idle,
            });
            self.animals_id += 1;
            vec![]
        };
        Ok((id, command))
    }
}
