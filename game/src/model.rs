use std::collections::{HashMap, HashSet};
use std::ptr::eq;

use crate::building::{
    Cell, GridId, GridKey, GridKind, Room, SurveyorId, SurveyorKey, SurveyorKind,
};
use crate::collections::{Dictionary, Shared};
use crate::inventory::{ContainerId, ContainerKey, ContainerKind, ItemId, ItemKey, ItemKind};
use crate::physics::{
    BarrierId, BarrierKey, BarrierKind, BodyId, BodyKey, BodyKind, SensorId, SensorKey, SensorKind,
    SpaceId, SpaceKey, SpaceKind,
};
use crate::planting::{LandId, LandKey, LandKind, PlantId, PlantKey, PlantKind};

#[derive(Default)]
pub struct Knowledge {
    pub trees: Dictionary<TreeKey, TreeKind>,
    pub farmlands: Dictionary<FarmlandKey, FarmlandKind>,
    pub farmers: Dictionary<FarmerKey, FarmerKind>,
    pub equipments: Dictionary<EquipmentKey, EquipmentKind>,
    pub crops: Dictionary<CropKey, CropKind>,
    // physics
    pub spaces: Dictionary<SpaceKey, SpaceKind>,
    pub bodies: Dictionary<BodyKey, BodyKind>,
    pub barriers: Dictionary<BarrierKey, BarrierKind>,
    pub sensors: Dictionary<SensorKey, SensorKind>,
    // inventory
    pub containers: Dictionary<ContainerKey, ContainerKind>,
    pub items: Dictionary<ItemKey, ItemKind>,
    // building
    pub grids: Dictionary<GridKey, GridKind>,
    pub surveyors: Dictionary<SurveyorKey, SurveyorKind>,
    // planting
    pub lands: Dictionary<LandKey, LandKind>,
    pub plants: Dictionary<PlantKey, PlantKind>,
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
    pub farmers_activity: HashMap<Farmer, Activity>,
    pub constructions: Vec<Construction>,
    pub constructions_id: usize,
    pub drops: Vec<Drop>,
    pub drops_id: usize,
    pub equipments: Vec<Equipment>,
    pub equipments_id: usize,
    pub crops: Vec<Crop>,
    pub crops_id: usize,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Universe {
    ActivityChanged {
        farmer: Farmer,
        activity: Activity,
    },
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
        holes: Vec<Vec<u8>>,
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
    CropAppeared {
        entity: Crop,
        impact: f32,
        thirst: f32,
        position: [f32; 2],
    },
    CropVanished(Crop),
    ConstructionAppeared {
        id: Construction,
        cell: [usize; 2],
    },
    ConstructionVanished {
        id: Construction,
    },
    EquipmentAppeared {
        entity: Equipment,
        position: [f32; 2],
    },
    EquipmentVanished(Equipment),
    ItemsAppeared {
        items: Vec<ItemRep>,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum UniverseError {
    FarmerActivityNotRegistered {
        farmer: Farmer,
    },
    FarmerActivityMismatch {
        expected: Activity,
        actual: Activity,
    },
}

impl UniverseDomain {
    pub fn load_farmlands(&mut self, farmlands: Vec<Farmland>, farmlands_id: usize) {
        self.farmlands_id = farmlands_id;
        self.farmlands.extend(farmlands);
    }

    pub fn load_farmers(&mut self, farmers: Vec<Farmer>, farmers_id: usize) {
        self.farmers_id = farmers_id;
        for farmer in &farmers {
            self.farmers_activity.insert(*farmer, Activity::Idle);
        }
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

    pub fn load_equipments(&mut self, equipments: Vec<Equipment>, equipments_id: usize) {
        self.equipments_id = equipments_id;
        self.equipments.extend(equipments);
    }

    pub fn load_crops(&mut self, crops: Vec<Crop>, crops_id: usize) {
        self.crops_id = crops_id;
        self.crops.extend(crops);
    }

    pub(crate) fn appear_construction(
        &mut self,
        container: ContainerId,
        grid: GridId,
        surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Vec<Universe> {
        self.constructions_id += 1;
        let construction = Construction {
            id: self.constructions_id,
            container,
            grid,
            surveyor,
            cell,
        };
        self.constructions.push(construction);
        vec![Universe::ConstructionAppeared {
            id: construction,
            cell,
        }]
    }

    pub(crate) fn vanish_construction(&mut self, id: Construction) -> Vec<Universe> {
        if let Some(index) = self
            .constructions
            .iter()
            .position(|construction| construction == &id)
        {
            self.constructions.remove(index);
            vec![Universe::ConstructionVanished { id }]
        } else {
            vec![]
        }
    }

    pub(crate) fn appear_crop(
        &mut self,
        key: CropKey,
        barrier: BarrierId,
        sensor: SensorId,
        plant: PlantId,
        impact: f32,
        thirst: f32,
        position: [f32; 2],
    ) -> Vec<Universe> {
        self.crops_id += 1;
        let entity = Crop {
            id: self.crops_id,
            key,
            plant,
            barrier,
            sensor,
        };
        self.crops.push(entity);
        vec![Universe::CropAppeared {
            entity,
            impact,
            thirst,
            position,
        }]
    }

    pub(crate) fn appear_equipment(
        &mut self,
        kind: EquipmentKey,
        purpose: Purpose,
        barrier: BarrierId,
        position: [f32; 2],
    ) -> Vec<Universe> {
        self.equipments_id += 1;
        let equipment = Equipment {
            id: self.equipments_id,
            kind,
            purpose,
            barrier,
        };
        self.equipments.push(equipment);
        vec![Universe::EquipmentAppeared {
            entity: equipment,
            position,
        }]
    }

    pub(crate) fn vanish_equipment(&mut self, id: Equipment) -> Vec<Universe> {
        if let Some(index) = self
            .equipments
            .iter()
            .position(|equipment| equipment == &id)
        {
            self.equipments.remove(index);
            vec![Universe::EquipmentVanished(id)]
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

    pub fn get_farmer_activity(&self, farmer: Farmer) -> Result<Activity, UniverseError> {
        self.farmers_activity
            .get(&farmer)
            .cloned()
            .ok_or(UniverseError::FarmerActivityNotRegistered { farmer })
    }

    pub fn ensure_activity(
        &self,
        farmer: Farmer,
        expected: Activity,
    ) -> Result<Activity, UniverseError> {
        let actual = self.get_farmer_activity(farmer)?;
        if actual != expected {
            Err(UniverseError::FarmerActivityMismatch { expected, actual })
        } else {
            Ok(actual)
        }
    }

    pub fn change_activity(&mut self, farmer: Farmer, activity: Activity) -> Vec<Universe> {
        self.farmers_activity.insert(farmer, activity);
        vec![Universe::ActivityChanged { farmer, activity }]
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
pub enum Activity {
    Idle,
    Delivery,
    Surveying {
        equipment: Equipment,
        selection: usize,
    },
    Instrumenting,
    Installing {
        item: ItemId,
    },
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
    pub surveyor: SurveyorId,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Deconstruction {
    pub id: usize,
    pub grid: GridId,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct EquipmentKey(pub usize);

pub enum PurposeDescription {
    Surveying { surveyor: SurveyorKey },
    Moisture { sensor: usize },
}

pub struct EquipmentKind {
    pub id: EquipmentKey,
    pub name: String,
    pub purpose: PurposeDescription,
    pub barrier: BarrierKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub enum Purpose {
    Surveying { surveyor: SurveyorId },
    Moisture { sensor: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Equipment {
    pub id: usize,
    pub kind: EquipmentKey,
    pub purpose: Purpose,
    pub barrier: BarrierId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Drop {
    pub id: usize,
    pub container: ContainerId,
    pub barrier: BarrierId,
}

// TODO: move to client (fix item appearance events)
#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct ItemRep {
    pub id: ItemId,
    pub kind: ItemKey,
    pub container: ContainerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct CropKey(pub usize);

pub struct CropKind {
    pub id: CropKey,
    pub name: String,
    pub plant: PlantKey,
    pub barrier: BarrierKey,
    pub sensor: SensorKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct Crop {
    pub id: usize,
    pub key: CropKey,
    pub plant: PlantId,
    pub barrier: BarrierId,
    pub sensor: SensorId,
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
