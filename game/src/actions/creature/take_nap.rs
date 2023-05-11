use crate::api::{ActionError, Event};
use crate::model::Creature;
use crate::raising::Behaviour;
use crate::{occur, Game};

impl Game {
    pub(crate) fn take_nap(&mut self, creature: Creature) -> Result<Vec<Event>, ActionError> {
        let change_behavior = self
            .raising
            .change_behaviour(creature.animal, Behaviour::Sleeping)?;
        let events = occur![change_behavior(),];
        Ok(events)
    }
}
