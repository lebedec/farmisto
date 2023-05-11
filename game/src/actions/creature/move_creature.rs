use crate::api::{ActionError, Event};
use crate::model::Creature;
use crate::Game;

impl Game {
    pub(crate) fn move_creature(
        &mut self,
        creature: Creature,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.physics.move_body2(creature.body, destination)?;
        Ok(vec![])
    }
}
