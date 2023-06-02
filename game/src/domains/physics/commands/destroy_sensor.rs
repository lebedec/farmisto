use crate::physics::{Physics, PhysicsDomain, PhysicsError, SensorId};

impl PhysicsDomain {
    pub fn destroy_sensor(
        &mut self,
        id: SensorId,
    ) -> Result<impl FnOnce() -> Vec<Physics> + '_, PhysicsError> {
        let sensor = self.get_sensor(id)?;
        let space = sensor.space;
        let command = move || {
            let index = self.sensors[space.0]
                .iter()
                .position(|sensor| sensor.id == id)
                .unwrap();
            let _sensor = self.sensors[space.0].remove(index);

            vec![]
        };
        Ok(command)
    }
}
