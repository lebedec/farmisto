use crate::api::{ActionError, Event};
use crate::model::{Door, Farmer, Universe};
use crate::{occur, Game};

impl Game {
    pub(crate) fn toggle_door(
        &mut self,
        _farmer: Farmer,
        door: Door,
    ) -> Result<Vec<Event>, ActionError> {
        let barrier = self.physics.get_barrier(door.barrier)?;
        let door_open = Universe::DoorChanged {
            entity: door,
            open: barrier.active,
        };
        let toggle_door = self.physics.change_barrier(barrier.id, !barrier.active)?;
        let events = occur![toggle_door(), door_open,];
        Ok(events)
    }
}
