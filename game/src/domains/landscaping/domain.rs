use std::collections::HashMap;

use crate::collections::{Sequence, Shared};
use crate::math::Rect;

pub type Place = [usize; 2];

pub struct Surface {}

impl Surface {
    pub const PLAINS: u8 = 0;
    pub const BASIN: u8 = 1;
}

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
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct LandId(pub usize);

pub struct Land {
    pub id: LandId,
    pub kind: Shared<LandKind>,
    pub moisture: Vec<f32>,
    pub moisture_capacity: Vec<f32>,
    pub surface: Vec<u8>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Landscaping {
    MoistureInspected {
        land: LandId,
        rect: Rect,
        moisture: Vec<f32>,
    },
    MoistureCapacityInspected {
        land: LandId,
        rect: Rect,
        moisture_capacity: Vec<f32>,
    },
    SurfaceInspected {
        land: LandId,
        rect: Rect,
        surface: Vec<u8>,
    }, // MoistureUpdate {
       //     land: LandId,
       //     moisture: LandMap,
       // },
       // MoistureUpdated {
       //     land: LandId,
       //     rect: [usize; 4],
       //     moisture: Vec<f32>,
       // },
       // MoistureCapacityUpdate {
       //     land: LandId,
       //     moisture_capacity: LandMap,
       // },
       // SurfaceUpdate {
       //     land: LandId,
       //     surface: LandMap,
       // },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum LandscapingError {
    LandNotFound {
        id: LandId,
    },
    InvalidLandSurface {
        id: LandId,
        expected: u8,
        actual: u8,
    },
    OutOfLand {
        id: LandId,
        place: Place,
    },
}
