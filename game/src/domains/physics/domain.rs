use core::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::collections::{Sequence, Shared};
use crate::math::Collider;

pub const MAX_SPACES: usize = 128;

pub struct PhysicsDomain {
    pub spaces: Vec<Space>,
    pub spaces_sequence: usize,
    pub bodies: Vec<Vec<Body>>,
    pub bodies_sequence: Sequence,
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
            bodies_sequence: Sequence::default(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpaceId(pub usize);

impl From<usize> for SpaceId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Into<usize> for SpaceId {
    fn into(self) -> usize {
        self.0
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BodyId(pub usize);

#[derive(Clone)]
pub struct Body {
    pub id: BodyId,
    pub kind: Shared<BodyKind>,
    pub position: [f32; 2],
    pub destination: [f32; 2],
    pub space: SpaceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BarrierKey(pub usize);

pub struct BarrierKind {
    pub id: BarrierKey,
    pub name: String,
    pub bounds: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BarrierId(pub usize);

#[derive(Clone)]
pub struct Barrier {
    pub id: BarrierId,
    pub kind: Shared<BarrierKind>,
    pub position: [f32; 2],
    pub space: SpaceId,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SensorKey(pub usize);

pub struct SensorKind {
    pub id: SensorKey,
    pub name: String,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SensorId(pub usize);

#[derive(Clone)]
pub struct Sensor {
    pub id: SensorId,
    pub kind: Shared<SensorKind>,
    pub position: [f32; 2],
    pub space: SpaceId,
    pub signals: Vec<[f32; 2]>,
    pub registered: HashSet<BodyId>,
}

#[derive(Serialize, Deserialize)]
pub enum Physics {
    BodyPositionChanged {
        id: BodyId,
        space: SpaceId,
        position: [f32; 2],
        destination: [f32; 2],
    },
    BarrierCreated {
        id: BarrierId,
        key: BarrierKey,
        space: SpaceId,
        position: [f32; 2],
        active: bool,
    },
    BarrierChanged {
        id: BarrierId,
        space: SpaceId,
        active: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum PhysicsError {
    SpaceNotFound { space: SpaceId },
    BodyNotFound { id: BodyId },
    BodyNotFoundAt { position: [f32; 2] },
    BarrierCreationOverlaps { other: BarrierId },
    BarrierNotFound { id: BarrierId },
    BarrierNotFoundAt { position: [f32; 2] },
    SensorNotFound { id: SensorId },
    HoleNotFound { hole: [usize; 2] },
    HoleAlreadyExists { hole: [usize; 2] },
    HoleCreationContainsBody { hole: [usize; 2] },
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
