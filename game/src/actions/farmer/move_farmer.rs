use crate::api::{ActionError, Event};
use crate::model::Farmer;
use crate::{emit, Game};

impl Game {
    pub(crate) fn move_farmer(
        &mut self,
        farmer: Farmer,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let move_body = self.physics.move_body(farmer.body, destination)?;
        emit![move_body()]
    }
}
