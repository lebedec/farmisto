use std::collections::{HashMap, HashSet};

use crate::building::{Cell, GridId, GridIndex, GridKey, Room, SurveyorId};
use crate::collections::Shared;
use crate::inventory::{ContainerId, ItemId, ItemKey};
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
    pub drops: Vec<Drop>,
    pub theodolites: Vec<Theodolite>,
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
    DropAppeared {
        drop: Drop,
        position: [f32; 2],
    },
    DropVanished(Drop),
    ConstructionAppeared {
        id: Construction,
        cell: [usize; 2],
    },
    ConstructionVanished(Construction),
    TheodoliteAppeared {
        entity: Theodolite,
        cell: [usize; 2],
    },
    TheodoliteVanished(Theodolite),
    ItemsAppeared {
        items: Vec<ItemView>,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum UniverseError {
    Nothing,
}

impl UniverseDomain {
    pub(crate) fn appear_construction(
        &mut self,
        container: ContainerId,
        grid: GridId,
        cell: [usize; 2],
    ) -> Vec<Universe> {
        let construction = Construction {
            id: self.constructions.len() + 1,
            container,
            grid,
            cell,
        };
        self.constructions.push(construction);
        vec![Universe::ConstructionAppeared {
            id: construction,
            cell,
        }]
    }

    pub(crate) fn vanish_construction(&mut self, construction: Construction) -> Vec<Universe> {
        if let Some(index) = self
            .constructions
            .iter()
            .position(|search| search == &construction)
        {
            self.constructions.remove(index);
            vec![Universe::ConstructionVanished(construction)]
        } else {
            vec![]
        }
    }

    pub fn appear_drop(
        &mut self,
        container: ContainerId,
        barrier: BarrierId,
        position: [f32; 2],
    ) -> Vec<Universe> {
        let drop = Drop {
            id: self.drops.len() + 1,
            container,
            barrier,
        };
        self.drops.push(drop);
        vec![Universe::DropAppeared { drop, position }]
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
    pub hands: ContainerId,
    pub backpack: ContainerId,
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
    pub grid: GridId,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Theodolite {
    pub id: usize,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Drop {
    pub id: usize,
    pub container: ContainerId,
    pub barrier: BarrierId,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct ItemView {
    pub id: ItemId,
    pub kind: ItemKey,
    pub container: ContainerId,
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

// Models:
//
// Entity - aggregate of domain objects (hold identifiers)
// EntityKind - aggregate of domain object kinds (defines object properties)
// Entity[Generated] - entity without EntityKind (defined dynamically in game run time)
// Value     - not domain object, used for action and events definition
// Event
// Action
//
// Universe - special|aggregate|root domain with entities

//  Domains:
//
// ObjectId - object identifies
// ObjectKey - memory efficient reference to object kind
// Object
// ObjectKind
// Object[Virtual] - not included in any entity (optimization purpose, e.g. 100500 inventory items)
// DomainEvent
