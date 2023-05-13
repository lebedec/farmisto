use crate::api::{ActionError, Event};
use crate::model::Creature;
use crate::raising::Behaviour;
use crate::{emit, Game};

impl Game {
    pub(crate) fn move_creature(
        &mut self,
        creature: Creature,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let trigger_behaviour =
            self.raising
                .trigger_behaviour(creature.animal, Behaviour::Walking, Behaviour::Idle)?;
        let move_body = self.physics.move_body(creature.body, destination)?;
        emit![trigger_behaviour(), move_body()]
    }
}
