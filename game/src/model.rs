use std::collections::{HashMap, HashSet};

use crate::building::{Cell, Grid, GridId, GridIndex, GridKey, Room, SurveyorId};
use crate::collections::Shared;
use crate::inventory::ContainerId;
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
pub enum Universe {
    BarrierHintAppeared {
        id: BarrierId,
        kind: BarrierKey,
        position: [f32; 2],
        bounds: [f32; 2],
    },
    TreeAppeared {
        tree: Tree,
        position: [f32; 2],
        growth: f32,
    },
    TreeVanished(Tree),
    FarmlandAppeared {
        farmland: Farmland,
        map: Vec<Vec<[f32; 2]>>,
        cells: Vec<Vec<Cell>>,
        rooms: Vec<Room>,
    },
    FarmlandVanished(Farmland),
    FarmerAppeared {
        farmer: Farmer,
        player: String,
        position: [f32; 2],
    },
    FarmerVanished(Farmer),
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum UniverseError {
    Nothing,
}

pub struct ConstructionAggregation<'action> {
    construction: Construction,
    constructions: &'action mut Vec<Construction>,
}

impl<'action> ConstructionAggregation<'action> {
    pub fn complete(self) -> Vec<Universe> {
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
    pub farmlands: HashSet<usize>,
    pub farmlands_to_delete: HashSet<usize>,
    pub trees: HashSet<usize>,
    pub trees_to_delete: HashSet<usize>,
    pub farmers: HashSet<usize>,
    pub farmers_to_delete: HashSet<usize>,
}

impl UniverseSnapshot {
    pub fn whole() -> Self {
        let mut snapshot = UniverseSnapshot::default();
        snapshot.whole = true;
        snapshot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct PlayerId(pub usize);

pub struct Player {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmerKey(pub usize);

pub struct FarmerKind {
    pub id: FarmerKey,
    pub name: String,
    pub body: BodyKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Farmer {
    pub id: usize,
    pub kind: FarmerKey,
    pub player: PlayerId,
    pub body: BodyId,
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
pub struct Tree {
    pub id: usize,
    pub kind: TreeKey,
    pub plant: PlantId,
    pub barrier: BarrierId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmlandKey(pub usize);

pub struct FarmlandKind {
    pub id: FarmlandKey,
    pub name: String,
    pub space: SpaceKey,
    pub land: LandKey,
    pub grid: GridKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Farmland {
    pub id: usize,
    pub kind: FarmlandKey,
    pub space: SpaceId,
    pub land: LandId,
    pub grid: GridId,
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
