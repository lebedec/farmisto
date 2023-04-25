use crate::collections::Shared;
use crate::math::VectorMath;
use crate::physics::Physics::BarrierCreated;
use crate::physics::{
    Barrier, BarrierId, BarrierKind, Physics, PhysicsDomain, PhysicsError, Sensor, SensorId,
    SensorKind, SpaceId,
};

impl PhysicsDomain {
    pub fn create_barrier_sensor<'operation>(
        &'operation mut self,
        space: SpaceId,
        barrier: &Shared<BarrierKind>,
        sensor: &Shared<SensorKind>,
        position: [f32; 2],
        overlapping: bool,
    ) -> Result<
        (
            BarrierId,
            SensorId,
            impl FnOnce() -> Vec<Physics> + 'operation,
        ),
        PhysicsError,
    > {
        if !overlapping {
            for barrier in &self.barriers[space.0] {
                if barrier.position.to_tile() == position.to_tile() {
                    return Err(PhysicsError::BarrierCreationOverlaps { other: barrier.id });
                }
            }
        }
        let barrier_id = BarrierId(self.barriers_sequence + 1);
        let barrier = Barrier {
            id: barrier_id,
            kind: barrier.clone(),
            position,
            space,
            active: true,
        };
        let sensor_id = SensorId(self.sensors_sequence + 1);
        let sensor = Sensor {
            id: sensor_id,
            kind: sensor.clone(),
            position,
            space,
            signals: vec![],
            registered: Default::default(),
        };
        let operation = move || {
            let events = vec![BarrierCreated {
                id: barrier.id,
                key: barrier.kind.id,
                space: barrier.space,
                position: barrier.position,
                active: barrier.active,
            }];
            self.barriers_sequence += 1;
            self.barriers[space.0].push(barrier);
            self.sensors_sequence += 1;
            self.sensors[space.0].push(sensor);
            events
        };
        Ok((barrier_id, sensor_id, operation))
    }
}
