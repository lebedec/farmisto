use crate::physics::{PhysicsDomain, PhysicsError, Sensor, SensorId};

impl PhysicsDomain {
    
    pub fn get_sensor(&self, id: SensorId) -> Result<&Sensor, PhysicsError> {
        for sensors in self.sensors.iter() {
            for sensor in sensors {
                if sensor.id == id {
                    return Ok(sensor);
                }
            }
        }
        return Err(PhysicsError::SensorNotFound { id });
    }
}