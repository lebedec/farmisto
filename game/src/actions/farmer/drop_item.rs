use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::math::TileMath;
use crate::model::{Farmer, Farmland};
use crate::{emit, Game};

impl Game {
    pub(crate) fn drop_item(
        &mut self,
        _farmland: Farmland,
        farmer: Farmer,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.ensure_target_reachable(farmer.body, tile.position())?;
        let body = self.physics.get_body(farmer.body)?;
        let space = body.space;
        let barrier_kind = self.known.barriers.find("<drop>")?;
        let (barrier, create_barrier) =
            self.physics
                .create_barrier(space, barrier_kind, tile.position(), true, false)?;
        let container_kind = self.known.containers.find("<drop>")?;
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let drop_item = self
            .inventory
            .drop_item(farmer.hands, container_kind, container)?;
        emit![
            create_barrier(),
            drop_item(),
            self.appear_stack(container, barrier)
        ]
    }
}
