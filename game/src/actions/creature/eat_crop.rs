use crate::api::{ActionError, Event};
use crate::model::{Creature, Crop};
use crate::raising::Behaviour;
use crate::{occur, Game};

impl Game {
    pub(crate) fn eat_crop(
        &mut self,
        creature: Creature,
        crop: Crop,
    ) -> Result<Vec<Event>, ActionError> {
        let bite = 0.3;
        let damage_plant = self.planting.damage_plant(crop.plant, bite)?;
        let feed_events = self.raising.feed_animal(creature.animal, bite)?();
        let trigger_behaviour =
            self.raising
                .trigger_behaviour(creature.animal, Behaviour::Eating, Behaviour::Idle)?;
        let events = occur![damage_plant(), feed_events, trigger_behaviour(),];
        Ok(events)
    }
}
