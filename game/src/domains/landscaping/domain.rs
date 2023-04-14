use std::collections::HashMap;

use crate::collections::{Sequence, Shared};

pub const LAND_WIDTH: usize = 128;
pub const LAND_HEIGHT: usize = 128;

pub type LandMap = [[u8; LAND_WIDTH]; LAND_HEIGHT];
pub type Place = [usize; 2];

pub struct LandscapingDomain {
    pub lands_update_interval: f32,
    pub lands_update: f32,
    pub lands_id: Sequence,
    pub lands: HashMap<LandId, Land>,
}

impl Default for LandscapingDomain {
    fn default() -> Self {
        Self {
            lands_update_interval: 0.5,
            lands_update: 0.0,
            lands_id: Sequence::default(),
            lands: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandKey(pub usize);

pub struct LandKind {
    pub id: LandKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct LandId(pub usize);

pub struct Land {
    pub id: LandId,
    pub kind: Shared<LandKind>,
    pub moisture: LandMap,
    pub moisture_capacity: LandMap,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Landscaping {
    MoistureUpdate {
        land: LandId,
        moisture: LandMap,
    },
    MoistureCapacityUpdate {
        land: LandId,
        moisture_capacity: LandMap,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum LandscapingError {
    LandNotFound { id: LandId },
    OutOfLand { id: LandId, place: Place },
}
