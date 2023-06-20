use crate::api::{ActionError, Event};
use crate::inventory::ItemId;
use crate::model::{Creature, Farmer, Stack};
use crate::{emit, Game};

impl Game {
    pub fn eat_food_from_stack(
        &mut self,
        creature: Creature,
        stack: Stack,
        _item: ItemId,
    ) -> Result<Vec<Event>, ActionError> {
        let position = self.physics.get_barrier(stack.barrier)?.position;
        self.ensure_target_reachable(creature.body, position)?;
        let decrease_item = self.inventory.decrease_container_item(stack.container)?;
        let feed_animal = self.raising.feed_animal(creature.animal, 0.1)?;
        let stop_body = self.physics.stop_body(creature.body)?;
        emit![stop_body(), decrease_item(), feed_animal()]
    }

    pub(crate) fn eat_food_from_hands(
        &mut self,
        creature: Creature,
        farmer: Farmer,
        _item: ItemId,
    ) -> Result<Vec<Event>, ActionError> {
        let position = self.physics.get_body(farmer.body)?.position;
        self.ensure_target_reachable(creature.body, position)?;
        let decrease_item = self.inventory.decrease_container_item(farmer.hands)?;
        let feed_animal = self.raising.feed_animal(creature.animal, 0.1)?;
        let stop_body = self.physics.stop_body(creature.body)?;
        emit![stop_body(), decrease_item(), feed_animal()]
    }
}
