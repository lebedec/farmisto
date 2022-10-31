use crate::collections::Shared;
use crate::physics::{BarrierId, BarrierKey, BodyId, BodyKey, SpaceId, SpaceKey};
use crate::planting::{LandId, LandKey, PlantId, PlantKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmerKey(pub usize);

pub struct FarmerKind {
    pub id: FarmerKey,
    pub name: String,
    pub body: BodyKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmerId(pub usize);

impl From<FarmerId> for BodyId {
    fn from(id: FarmerId) -> Self {
        Self(id.0)
    }
}

pub struct Farmer {
    pub id: FarmerId,
    pub kind: Shared<FarmerKind>,
    pub player: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct TreeKey(pub usize);

pub struct TreeKind {
    pub id: TreeKey,
    pub name: String,
    pub barrier: BarrierKey,
    pub plant: PlantKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct TreeId(pub usize);

impl From<TreeId> for BarrierId {
    fn from(id: TreeId) -> Self {
        Self(id.0)
    }
}

impl From<TreeId> for PlantId {
    fn from(id: TreeId) -> Self {
        Self(id.0)
    }
}

pub struct Tree {
    pub id: TreeId,
    pub kind: Shared<TreeKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmlandKey(pub usize);

pub struct FarmlandKind {
    pub id: FarmlandKey,
    pub name: String,
    pub space: SpaceKey,
    pub land: LandKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmlandId(pub usize);

impl From<FarmlandId> for SpaceId {
    fn from(id: FarmlandId) -> Self {
        Self(id.0)
    }
}

impl From<FarmlandId> for LandId {
    fn from(id: FarmlandId) -> Self {
        Self(id.0)
    }
}

pub struct Farmland {
    pub id: FarmlandId,
    pub kind: Shared<FarmlandKind>,
}
