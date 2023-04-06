use std::collections::HashMap;

#[derive(Default)]
pub struct AssemblingDomain {
    pub placements_id: usize,
    pub placements: HashMap<PlacementId, Placement>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Assembling {
    PlacementUpdated {
        placement: PlacementId,
        rotation: Rotation,
        pivot: [usize; 2],
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum AssemblingError {
    PlacementNotFound { id: PlacementId },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode, serde::Deserialize,
)]
pub enum Rotation {
    A000,
    A090,
    A180,
    A270,
}

impl Rotation {
    pub fn index(&self) -> usize {
        match self {
            Rotation::A000 => 0,
            Rotation::A090 => 1,
            Rotation::A180 => 2,
            Rotation::A270 => 3,
        }
    }
    
    pub fn from_index(index: u8) -> Rotation {
        match index {
            0 => Rotation::A000,
            1 => Rotation::A090,
            2 => Rotation::A180,
            3 => Rotation::A270,
            _ => Rotation::A000
        }
    }
    
    pub fn next(&self) -> Rotation {
        match self {
            Rotation::A000 => Rotation::A090,
            Rotation::A090 => Rotation::A180,
            Rotation::A180 => Rotation::A270,
            Rotation::A270 => Rotation::A000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct PlacementId(pub usize);

#[derive(Debug, Clone)]
pub struct Placement {
    pub id: PlacementId,
    pub rotation: Rotation,
    pub pivot: [usize; 2],
}
