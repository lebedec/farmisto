use core::fmt::{Debug, Formatter};

use crate::collections::Shared;
use crate::math::Collider;

pub const MAX_SPACES: usize = 128;

pub struct PhysicsDomain {
    pub spaces: Vec<Space>,
    pub spaces_sequence: usize,
    pub bodies: Vec<Vec<Body>>,
    pub bodies_sequence: usize,
    pub barriers: Vec<Vec<Barrier>>,
    pub barriers_sequence: usize,
    pub sensors: Vec<Vec<Sensor>>,
    pub sensors_sequence: usize,
}

impl Default for PhysicsDomain {
    fn default() -> Self {
        Self {
            spaces: vec![],
            spaces_sequence: 0,
            bodies: vec![vec![]; MAX_SPACES],
            bodies_sequence: 0,
            barriers: vec![vec![]; MAX_SPACES],
            barriers_sequence: 0,
            sensors: vec![vec![]; MAX_SPACES],
            sensors_sequence: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceKey(pub usize);

pub struct SpaceKind {
    pub id: SpaceKey,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SpaceId(pub usize);

pub struct Space {
    pub id: SpaceId,
    pub kind: Shared<SpaceKind>,
    pub holes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyKey(pub usize);

pub struct BodyKind {
    pub id: BodyKey,
    pub name: String,
    pub speed: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct BodyId(pub usize);

#[derive(Clone)]
pub struct Body {
    pub id: BodyId,
    pub kind: Shared<BodyKind>,
    pub position: [f32; 2],
    pub destination: [f32; 2],
    pub space: SpaceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct BarrierKey(pub usize);

pub struct BarrierKind {
    pub id: BarrierKey,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct BarrierId(pub usize);

#[derive(Clone)]
pub struct Barrier {
    pub id: BarrierId,
    pub kind: Shared<BarrierKind>,
    pub position: [f32; 2],
    pub space: SpaceId,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SensorKey(pub usize);

pub struct SensorKind {
    pub id: SensorKey,
    pub name: String,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SensorId(pub usize);

#[derive(Clone)]
pub struct Sensor {
    pub id: SensorId,
    pub kind: Shared<SensorKind>,
    pub position: [f32; 2],
    pub space: SpaceId,
    pub signals: Vec<[f32; 2]>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum Physics {
    BodyPositionChanged {
        id: BodyId,
        space: SpaceId,
        position: [f32; 2],
    },
    BarrierCreated {
        id: BarrierId,
        space: SpaceId,
        position: [f32; 2],
        bounds: [f32; 2],
    },
    BarrierDestroyed {
        id: BarrierId,
        space: SpaceId,
        position: [f32; 2],
    },
    SpaceUpdated {
        id: SpaceId,
        holes: Vec<Vec<u8>>,
    },
}

impl Debug for Physics {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Physics::SpaceUpdated { id, .. } => {
                f.debug_struct("SpaceUpdated").field("id", id).finish()
            }
            other => Debug::fmt(other, f),
        }
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PhysicsError {
    SpaceNotFound { space: SpaceId },
    BodyNotFound { id: BodyId },
    BarrierCreationOverlaps { other: BarrierId },
    BarrierNotFound { id: BarrierId },
    SensorNotFound { id: SensorId },
    HoleNotFound { hole: [usize; 2] },
    HoleAlreadyExists { hole: [usize; 2] },
    HoleCreationContainsBody { hole: [usize; 2] },
}

impl PhysicsDomain {
    pub fn load_spaces(&mut self, spaces: Vec<Space>, sequence: usize) {
        self.spaces_sequence = sequence;
        self.spaces.extend(spaces);
    }

    pub fn load_bodies(&mut self, bodies: Vec<Body>, sequence: usize) {
        self.bodies_sequence = sequence;
        for body in bodies {
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

    pub fn get_body_mut(&mut self, id: BodyId) -> Result<&mut Body, PhysicsError> {
        for bodies in self.bodies.iter_mut() {
            for body in bodies {
                if body.id == id {
                    return Ok(body);
                }
            }
        }
        return Err(PhysicsError::BodyNotFound { id });
    }

    pub fn get_space(&self, id: SpaceId) -> Result<&Space, PhysicsError> {
        self.spaces
            .iter()
            .find(|space| space.id == id)
            .ok_or(PhysicsError::SpaceNotFound { space: id })
    }

    pub fn get_space_mut(&mut self, id: SpaceId) -> Result<&mut Space, PhysicsError> {
        self.spaces
            .iter_mut()
            .find(|space| space.id == id)
            .ok_or(PhysicsError::SpaceNotFound { space: id })
    }

    pub fn get_body(&self, id: BodyId) -> Result<&Body, PhysicsError> {
        for bodies in self.bodies.iter() {
            for body in bodies {
                if body.id == id {
                    return Ok(body);
                }
            }
        }
        return Err(PhysicsError::BodyNotFound { id });
    }

    pub fn get_barrier(&self, id: BarrierId) -> Result<&Barrier, PhysicsError> {
        for barriers in self.barriers.iter() {
            for barrier in barriers {
                if barrier.id == id {
                    return Ok(barrier);
                }
            }
        }
        return Err(PhysicsError::BarrierNotFound { id });
    }

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

pub struct Hole {
    pub position: [f32; 2],
    pub bounds: [f32; 2],
}

impl Collider for Hole {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn bounds(&self) -> [f32; 2] {
        self.bounds
    }
}

impl Collider for &Body {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn bounds(&self) -> [f32; 2] {
        [self.kind.radius, self.kind.radius]
    }
}

impl Collider for Barrier {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    #[inline]
    fn bounds(&self) -> [f32; 2] {
        if self.active {
            self.kind.bounds
        } else {
            [0.0, 0.0]
        }
    }
}
