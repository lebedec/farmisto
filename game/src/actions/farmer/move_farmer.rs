use crate::api::{ActionError, Event};
use crate::Game;
use crate::model::Farmer;

impl Game {
    pub(crate) fn move_farmer(
        &mut self,
        farmer: Farmer,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.physics.move_body2(farmer.body, destination)?;
        Ok(vec![])
    }
}