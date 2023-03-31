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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct PlacementId(pub usize);

#[derive(Debug, Clone)]
pub struct Placement {
    pub id: PlacementId,
    pub rotation: Rotation,
    pub pivot: [usize; 2],
}
