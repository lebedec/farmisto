use crate::api::{ActionError, Event};
use crate::inventory::{ContainerId, Item, ItemId};
use crate::math::TileMath;
use crate::model::{Farmer, Farmland};
use crate::{emit, occur, Game};

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
        let create_container = self
            .inventory
            .add_empty_container(container, &container_kind)?;
        // container not found !!!! (not created because)
        let move_item = self.inventory.pop_item(farmer.hands, container)?;

        emit![
            create_barrier(),
            create_container(),
            move_item(),
            self.create_stack(container, barrier)
        ]
    }
}

struct Transaction {
    create: Vec<Item>,
    update: Vec<Item>,
    delete: Vec<ItemId>,
}

struct Transfer {}
