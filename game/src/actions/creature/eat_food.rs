use crate::api::{ActionError, Event};
use crate::inventory::{ContainerId, ItemId};
use crate::model::{Creature, Farmer, Stack};
use crate::{occur, Game};
use log::error;

impl Game {
    pub fn eat_food_from_stack(
        &mut self,
        creature: Creature,
        stack: Stack,
        item: ItemId,
    ) -> Result<Vec<Event>, ActionError> {
        let position = self.physics.get_barrier(stack.barrier)?.position;
        self.ensure_target_reachable(creature.body, position)?;
        let decrease_item = self.inventory.decrease_container_item(stack.container)?;
        let feed_animal = self.raising.feed_animal(creature.animal, 0.1)?;
        let events = occur![decrease_item(), feed_animal(),];
        Ok(events)
    }

    pub(crate) fn eat_food_from_hands(
        &mut self,
        creature: Creature,
        farmer: Farmer,
        item: ItemId,
    ) -> Result<Vec<Event>, ActionError> {
        let position = self.physics.get_body(farmer.body)?.position;
        self.ensure_target_reachable(creature.body, position)?;
        let decrease_item = self.inventory.decrease_container_item(farmer.hands)?;
        let feed_animal = self.raising.feed_animal(creature.animal, 0.1)?;
        let events = occur![decrease_item(), feed_animal(),];
        Ok(events)
    }
}
