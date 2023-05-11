use crate::api::{ActionError, Event};
use crate::inventory::ItemId;
use crate::model::Creature;
use crate::raising::Behaviour;
use crate::{occur, Game};

impl Game {
    pub(crate) fn eat_food(
        &mut self,
        creature: Creature,
        food: ItemId,
    ) -> Result<Vec<Event>, ActionError> {
        // TODO: transactional
        let feed_events = self.raising.feed_animal(creature.animal, 0.1)?();
        let trigger_behaviour =
            self.raising
                .trigger_behaviour(creature.animal, Behaviour::Eating, Behaviour::Idle)?;
        // self.ensure_target_reachable(creature.body, )
        // let eat_food = self.inventory.decrease_container_item()
        let events = occur![feed_events, trigger_behaviour(),];
        Ok(events)
    }
}
