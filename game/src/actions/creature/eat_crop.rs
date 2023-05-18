use crate::api::{ActionError, Event};
use crate::model::{Creature, Crop};
use crate::{emit, Game};

impl Game {
    pub(crate) fn eat_crop(
        &mut self,
        creature: Creature,
        crop: Crop,
    ) -> Result<Vec<Event>, ActionError> {
        let bite = 0.3;
        let damage_plant = self.planting.damage_plant(crop.plant, bite)?;
        let feed_animal = self.raising.feed_animal(creature.animal, bite)?;
        let stop_body = self.physics.stop_body(creature.body)?;
        emit![stop_body(), damage_plant(), feed_animal()]
    }
}
