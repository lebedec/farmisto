use crate::api::{ActionError, Event};
use crate::inventory::{ItemId};

use crate::math::{TileMath};
use crate::model::{Activity, Corpse, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn collect_corpse(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        corpse: Corpse,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        let position = self.physics.get_barrier(corpse.barrier)?.position;
        self.ensure_target_reachable(farmer.body, position)?;

        let corpse_kind = self.known.corpses.get(corpse.key)?;
        let destroy_barrier = self.physics.destroy_barrier(corpse.barrier)?;
        let item = self.inventory.items_id.introduce().one(ItemId);
        let create_item = self
            .inventory
            .create_item(item, &corpse_kind.item, farmer.hands, 1)?;

        let events = occur![
            self.universe.vanish_corpse(corpse),
            destroy_barrier(),
            create_item(),
            self.universe.change_activity(farmer, Activity::Usage),
        ];

        Ok(events)
    }
}
