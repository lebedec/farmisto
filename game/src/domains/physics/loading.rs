use crate::physics::{Barrier, Body, PhysicsDomain, Sensor, Space};

impl PhysicsDomain {
    pub fn load_spaces(&mut self, spaces: Vec<Space>, sequence: usize) {
        self.spaces_sequence = sequence;
        self.spaces.extend(spaces);
    }

    pub fn load_bodies(&mut self, bodies: Vec<Body>) {
        for body in bodies {
            self.bodies_sequence.register(body.id.0);
            self.bodies[body.space.0].push(body);
        }
    }

    pub fn load_barriers(&mut self, barriers: Vec<Barrier>, sequence: usize) {
        self.barriers_sequence = sequence;
        for barrier in barriers {
            self.barriers[barrier.space.0].push(barrier);
        }
    }

    pub fn load_sensors(&mut self, sensors: Vec<Sensor>, sequence: usize) {
        self.sensors_sequence = sequence;
        for sensor in sensors {
            self.sensors[sensor.space.0].push(sensor);
        }
    }
}
