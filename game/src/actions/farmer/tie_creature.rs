use crate::api::{ActionError, Event};
use crate::{emit, Game};
use crate::inventory::FunctionsQuery;
use crate::model::{Activity, Creature, Farmer};
use crate::raising::AnimalId;

impl Game {
    pub(crate) fn tie_creature(
        &mut self,
        farmer: Farmer,
        creature: Creature,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        item.kind.functions.as_tether()?;
        let target = self.physics.get_body(creature.body)?.position;
        self.ensure_target_reachable(farmer.body, target)?;
        
        let activity = Activity::Tethering { creature };
        let tie_animal = self.raising.tie_animal(farmer.tether, creature.animal)?;
        
        emit![
            tie_animal(),
            self.universe.change_activity(farmer, activity)
        ]
    }
}