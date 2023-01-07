use std::collections::{HashMap, HashSet};
use crate::building::PlatformId;

use crate::collections::Shared;
use crate::physics::{BarrierId, BarrierKey, BodyId, BodyKey, SpaceId, SpaceKey};
use crate::planting::{LandId, LandKey, PlantId, PlantKey};

#[derive(Default)]
pub struct KnowledgeBase {
    pub trees: HashMap<TreeKey, Shared<TreeKind>>,
    pub farmlands: HashMap<FarmlandKey, Shared<FarmlandKind>>,
    pub farmers: HashMap<FarmerKey, Shared<FarmerKind>>,
}

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known: KnowledgeBase,
    pub farmlands: Vec<Farmland>,
    pub trees: Vec<Tree>,
    pub farmers: Vec<Farmer>,
}

#[derive(Default)]
pub struct UniverseSnapshot {
    pub whole: bool,
    pub farmlands: HashSet<FarmlandId>,
    pub farmlands_to_delete: HashSet<FarmlandId>,
    pub trees: HashSet<TreeId>,
    pub trees_to_delete: HashSet<TreeId>,
    pub farmers: HashSet<FarmerId>,
    pub farmers_to_delete: HashSet<FarmerId>,
}

impl UniverseSnapshot {
    pub fn whole() -> Self {
        let mut snapshot = UniverseSnapshot::default();
        snapshot.whole = true;
        snapshot
    }
}

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

impl From<FarmlandId> for PlatformId {
    fn from(id: FarmlandId) -> Self {
        Self(id.0)
    }
}

impl From<LandId> for FarmlandId {
    fn from(id: LandId) -> Self {
        Self(id.0)
    }
}

impl From<PlatformId> for FarmlandId {
    fn from(id: PlatformId) -> Self {
        Self(id.0)
    }
}

pub struct Farmland {
    pub id: FarmlandId,
    pub kind: Shared<FarmlandKind>,
}
