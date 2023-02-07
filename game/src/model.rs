use std::collections::{HashMap, HashSet};

use crate::building::{Cell, GridId, GridKey, Room};
use crate::collections::{Dictionary, Shared};
use crate::inventory::{ContainerId, ContainerKey, ContainerKind, ItemId, ItemKey, ItemKind};
use crate::physics::{
    BarrierId, BarrierKey, BarrierKind, BodyId, BodyKey, BodyKind, SpaceId, SpaceKey, SpaceKind,
};
use crate::planting::{LandId, LandKey, PlantId, PlantKey};

#[derive(Default)]
pub struct Knowledge {
    pub trees: Dictionary<TreeKey, TreeKind>,
    pub farmlands: Dictionary<FarmlandKey, FarmlandKind>,
    pub farmers: Dictionary<FarmerKey, FarmerKind>,
    // physics
    pub spaces: Dictionary<SpaceKey, SpaceKind>,
    pub bodies: Dictionary<BodyKey, BodyKind>,
    pub barriers: Dictionary<BarrierKey, BarrierKind>,
    // inventory
    pub containers: Dictionary<ContainerKey, ContainerKind>,
    pub items: Dictionary<ItemKey, ItemKind>,
}

#[derive(Default)]
pub struct UniverseDomain {
    pub id: usize,
    pub farmlands: Vec<Farmland>,
    pub farmlands_id: usize,
    pub trees: Vec<Tree>,
    trees_id: usize,
    pub farmers: Vec<Farmer>,
    pub farmers_id: usize,
    pub constructions: Vec<Construction>,
    pub constructions_id: usize,
    pub drops: Vec<Drop>,
    pub drops_id: usize,
    pub theodolites: Vec<Theodolite>,
    pub theodolites_id: usize,
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
    pub fn load_farmlands(&mut self, farmlands: Vec<Farmland>, farmlands_id: usize) {
        self.farmlands_id = farmlands_id;
        self.farmlands.extend(farmlands);
    }

    pub fn load_farmers(&mut self, farmers: Vec<Farmer>, farmers_id: usize) {
        self.farmers_id = farmers_id;
        self.farmers.extend(farmers);
    }

    pub fn load_trees(&mut self, trees: Vec<Tree>, trees_id: usize) {
        self.trees_id = trees_id;
        self.trees.extend(trees);
    }

    pub fn load_constructions(
        &mut self,
        constructions: Vec<Construction>,
        constructions_id: usize,
    ) {
        self.constructions_id = constructions_id;
        self.constructions.extend(constructions);
    }

    pub fn load_drops(&mut self, drops: Vec<Drop>, drops_id: usize) {
        self.drops_id = drops_id;
        self.drops.extend(drops);
    }

    pub fn load_theodolites(&mut self, theodolites: Vec<Theodolite>, theodolites_id: usize) {
        self.theodolites_id = theodolites_id;
        self.theodolites.extend(theodolites);
    }

    pub(crate) fn appear_construction(
        &mut self,
        container: ContainerId,
        grid: GridId,
        cell: [usize; 2],
    ) -> Vec<Universe> {
        self.constructions_id += 1;
        let construction = Construction {
            id: self.constructions_id,
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
        self.drops_id += 1;
        let drop = Drop {
            id: self.drops_id,
            container,
            barrier,
        };
        self.drops.push(drop);
        vec![Universe::DropAppeared { drop, position }]
    }

    pub fn vanish_drop(&mut self, drop: Drop) -> Vec<Universe> {
        let index = self
            .drops
            .iter()
            .position(|search| search.id == drop.id)
            .unwrap();
        self.drops.remove(index);
        vec![Universe::DropVanished(drop)]
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
