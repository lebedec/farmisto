use crate::api::Event;
use crate::building::{GridId, GridIndex, Surveyor, SurveyorId};
use std::collections::{HashMap, HashSet};

use crate::collections::Shared;
use crate::inventory::{ContainerId, ItemId};
use crate::physics::{BarrierId, BarrierKey, BodyId, BodyKey, SpaceId, SpaceKey};
use crate::planting::{LandId, LandKey, PlantId, PlantKey};

#[derive(Default)]
pub struct KnowledgeBase {
    pub trees: HashMap<TreeKey, Shared<TreeKind>>,
    pub farmlands: HashMap<FarmlandKey, Shared<FarmlandKind>>,
    pub farmers: HashMap<FarmerKey, Shared<FarmerKind>>,
}

#[derive(Default)]
pub struct UniverseDomain {
    pub id: usize,
    pub known: KnowledgeBase,
    pub farmlands: Vec<Farmland>,
    pub trees: Vec<Tree>,
    pub farmers: Vec<Farmer>,
    pub constructions: Vec<Construction>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum UniverseError {}

pub struct ConstructionAggregation<'action> {
    construction: Construction,
    constructions: &'action mut Vec<Construction>,
}

impl<'action> ConstructionAggregation<'action> {
    pub fn complete(self) -> Vec<Event> {
        let events = vec![];
        self.constructions.push(self.construction);
        events
    }
}

impl UniverseDomain {
    pub(crate) fn aggregate_to_construction(
        &mut self,
        container: ContainerId,
        cell: GridIndex,
    ) -> Result<ConstructionAggregation, UniverseError> {
        Ok(ConstructionAggregation {
            construction: Construction {
                id: self.constructions.len(),
                container,
                cell,
            },
            constructions: &mut self.constructions,
        })
    }
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

impl From<FarmlandId> for GridId {
    fn from(id: FarmlandId) -> Self {
        Self(id.0)
    }
}

impl From<LandId> for FarmlandId {
    fn from(id: LandId) -> Self {
        Self(id.0)
    }
}

impl From<GridId> for FarmlandId {
    fn from(id: GridId) -> Self {
        Self(id.0)
    }
}

pub struct Farmland {
    pub id: FarmlandId,
    pub kind: Shared<FarmlandKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Construction {
    pub id: usize,
    pub container: ContainerId,
    pub cell: GridIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Theodolite {
    pub id: usize,
    pub surveyor: SurveyorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, bincode::Encode, bincode::Decode)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
