use crate::{Game, occur, position_of};
use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::model::{Activity, Farmer};

impl Game {
    pub(crate) fn drop_item(&mut self, farmer: Farmer, tile: [usize; 2]) -> Result<Vec<Event>, ActionError> {
        let hands = self.inventory.get_container(farmer.hands)?;
        let is_last_item = hands.items.len() <= 1;
        let body = self.physics.get_body(farmer.body)?;
        let space = body.space;
        let barrier_kind = self.known.barriers.find("<drop>").unwrap();
        let position = position_of(tile);
        let (barrier, create_barrier) =
            self.physics
                .create_barrier(space, barrier_kind, position, true, false)?;
        let container_kind = self.known.containers.find("<drop>").unwrap();
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let extract_item =
            self.inventory
                .extract_item(farmer.hands, -1, container, container_kind)?;
        let activity = if is_last_item {
            self.universe.change_activity(farmer, Activity::Idle)
        } else {
            vec![]
        };
        let events = occur![
            create_barrier(),
            extract_item(),
            self.appear_stack(container, barrier),
            activity,
        ];
        Ok(events)
    }
}