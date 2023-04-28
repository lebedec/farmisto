use crate::assembling::{PlacementId, Rotation};
use std::collections::{HashMap, HashSet};

use crate::building::{
    Cell, GridId, GridKey, GridKind, Marker, Room, SurveyorId, SurveyorKey, SurveyorKind,
};
use crate::collections::{Dictionary, Shared};
use crate::inventory::{ContainerId, ContainerKey, ContainerKind, ItemId, ItemKey, ItemKind};
use crate::landscaping::{LandId, LandKey, LandKind};
use crate::physics::{
    BarrierId, BarrierKey, BarrierKind, BodyId, BodyKey, BodyKind, SensorId, SensorKey, SensorKind,
    SpaceId, SpaceKey, SpaceKind,
};
use crate::planting::{PlantId, PlantKey, PlantKind, SoilId, SoilKey, SoilKind};
use crate::raising::{AnimalId, AnimalKey, AnimalKind};
use crate::timing::{CalendarId, CalendarKey, CalendarKind, Sequence};
use crate::working::{DeviceId, DeviceKey, DeviceKind};

#[derive(Default)]
pub struct Knowledge {
    pub trees: Dictionary<TreeKey, TreeKind>,
    pub farmlands: Dictionary<FarmlandKey, FarmlandKind>,
    pub farmers: Dictionary<FarmerKey, FarmerKind>,
    pub equipments: Dictionary<EquipmentKey, EquipmentKind>,
    pub assembly: Dictionary<AssemblyKey, AssemblyKind>,
    pub crops: Dictionary<CropKey, CropKind>,
    pub creatures: Dictionary<CreatureKey, CreatureKind>,
    pub doors: Dictionary<DoorKey, DoorKind>,
    pub cementers: Dictionary<CementerKey, CementerKind>,
    pub composters: Dictionary<ComposterKey, ComposterKind>,
    pub rests: Dictionary<RestKey, RestKind>,
    // timing
    pub calendars: Dictionary<CalendarKey, CalendarKind>,
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
    pub soils: Dictionary<SoilKey, SoilKind>,
    pub plants: Dictionary<PlantKey, PlantKind>,
    // raising
    pub animals: Dictionary<AnimalKey, AnimalKind>,
    // working
    pub devices: Dictionary<DeviceKey, DeviceKind>,
    // landscaping
    pub lands: Dictionary<LandKey, LandKind>,
}

#[derive(Default)]
pub struct UniverseDomain {
    pub id: usize,
    pub farmlands: Vec<Farmland>,
    pub farmlands_id: usize,
    pub trees: Vec<Tree>,
    pub trees_id: usize,
    pub farmers: Vec<Farmer>,
    pub farmers_id: usize,
    pub farmers_activity: HashMap<Farmer, Activity>,
    pub constructions: Vec<Construction>,
    pub constructions_id: usize,
    pub stacks: Vec<Stack>,
    pub stacks_id: usize,
    pub equipments: Vec<Equipment>,
    pub equipments_id: usize,
    pub crops: Vec<Crop>,
    pub crops_id: usize,
    pub creatures: Vec<Creature>,
    pub creatures_id: usize,
    pub assembly: Vec<Assembly>,
    pub assembly_id: usize,
    pub doors: Vec<Door>,
    pub doors_id: usize,
    pub rests: Vec<Rest>,
    pub rests_id: usize,
    pub cementers: Vec<Cementer>,
    pub cementers_id: usize,
    pub composters: Vec<Composter>,
    pub composters_id: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Universe {
    ActivityChanged {
        farmer: Farmer,
        activity: Activity,
    },
    TreeAppeared {
        tree: Tree,
        position: [f32; 2],
        growth: f32,
    },
    TreeVanished(Tree),
    FarmlandAppeared {
        farmland: Farmland,
        // moisture: LandMap,
        // moisture_capacity: LandMap,
        cells: Vec<Vec<Cell>>,
        rooms: Vec<Room>,
        holes: Vec<Vec<u8>>,
        season: u8,
        season_day: f32,
        times_of_day: f32,
    },
    FarmlandVanished(Farmland),
    FarmerAppeared {
        farmer: Farmer,
        player: String,
        position: [f32; 2],
    },
    FarmerVanished(Farmer),
    StackAppeared {
        stack: Stack,
        position: [f32; 2],
    },
    StackVanished(Stack),
    CropAppeared {
        entity: Crop,
        impact: f32,
        thirst: f32,
        hunger: f32,
        growth: f32,
        health: f32,
        fruits: f32,
        position: [f32; 2],
    },
    CropVanished(Crop),
    CreatureAppeared {
        entity: Creature,
        space: SpaceId,
        health: f32,
        hunger: f32,
        position: [f32; 2],
    },
    CreatureEats {
        entity: Creature,
    },
    CreatureVanished(Creature),
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
        items: Vec<ItemData>,
    },
    AssemblyAppeared {
        entity: Assembly,
        rotation: Rotation,
        pivot: [usize; 2],
        valid: bool,
    },
    AssemblyUpdated {
        entity: Assembly,
        rotation: Rotation,
        pivot: [usize; 2],
    },
    AssemblyVanished(Assembly),
    DoorAppeared {
        entity: Door,
        open: bool,
        rotation: Rotation,
        position: [f32; 2],
    },
    DoorChanged {
        entity: Door,
        open: bool,
    },
    DoorVanished(Door),
    RestAppeared {
        entity: Rest,
        rotation: Rotation,
        position: [f32; 2],
    },
    RestVanished(Rest),
    CementerAppeared {
        entity: Cementer,
        rotation: Rotation,
        position: [f32; 2],
        enabled: bool,
        broken: bool,
        input: bool,
        output: bool,
        progress: f32,
    },
    CementerVanished(Cementer),
    ComposterInspected {
        entity: Composter,
        rotation: Rotation,
        position: [f32; 2],
        enabled: bool,
        broken: bool,
        input: bool,
        output: bool,
        progress: f32,
    },
    ComposterVanished(Composter),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum UniverseError {
    FarmlandBySpaceNotFound {
        space: SpaceId,
    },
    PlayerFarmerNotFound {
        player: PlayerId,
    },
    FarmerActivityNotRegistered {
        farmer: Farmer,
    },
    FarmerActivityMismatch {
        expected: Activity,
        actual: Activity,
    },
    ActivityMismatch,
    ModeInvalidEnumeration {
        value: u8,
    },
}

impl UniverseDomain {
    pub fn load_farmlands(&mut self, farmlands: Vec<Farmland>, farmlands_id: usize) {
        self.farmlands_id = farmlands_id;
        self.farmlands.extend(farmlands);
    }

    pub fn get_player_farmer(&self, player: PlayerId) -> Result<Farmer, UniverseError> {
        self.farmers
            .iter()
            .find(|farmer| farmer.player == player)
            .cloned()
            .ok_or(UniverseError::PlayerFarmerNotFound { player })
    }

    pub fn get_farmland_by_space(&self, space: SpaceId) -> Result<Farmland, UniverseError> {
        self.farmlands
            .iter()
            .find(|farmland| farmland.space == space)
            .cloned()
            .ok_or(UniverseError::FarmlandBySpaceNotFound { space })
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

    pub fn load_stacks(&mut self, stacks: Vec<Stack>, stacks_id: usize) {
        self.stacks_id = stacks_id;
        self.stacks.extend(stacks);
    }

    pub fn load_equipments(&mut self, equipments: Vec<Equipment>, equipments_id: usize) {
        self.equipments_id = equipments_id;
        self.equipments.extend(equipments);
    }

    pub fn load_crops(&mut self, crops: Vec<Crop>, crops_id: usize) {
        self.crops_id = crops_id;
        self.crops.extend(crops);
    }

    pub fn load_creatures(&mut self, creatures: Vec<Creature>, creatures_id: usize) {
        self.creatures_id = creatures_id;
        self.creatures.extend(creatures);
    }

    pub fn load_assembly(&mut self, assembly: Vec<Assembly>, assembly_id: usize) {
        self.assembly_id = assembly_id;
        self.assembly.extend(assembly);
    }

    pub fn load_doors(&mut self, doors: Vec<Door>, doors_id: usize) {
        self.doors_id = doors_id;
        self.doors.extend(doors);
    }

    pub fn load_rests(&mut self, rests: Vec<Rest>, rests_id: usize) {
        self.rests_id = rests_id;
        self.rests.extend(rests);
    }

    pub fn load_cementers(&mut self, cementers: Vec<Cementer>, cementers_id: usize) {
        self.cementers_id = cementers_id;
        self.cementers.extend(cementers);
    }

    pub fn load_composters(&mut self, composters: Vec<Composter>, composters_id: usize) {
        self.composters_id = composters_id;
        self.composters.extend(composters);
    }

    pub(crate) fn appear_construction(
        &mut self,
        container: ContainerId,
        grid: GridId,
        surveyor: SurveyorId,
        marker: Marker,
        cell: [usize; 2],
    ) -> Vec<Universe> {
        self.constructions_id += 1;
        let construction = Construction {
            id: self.constructions_id,
            container,
            grid,
            surveyor,
            marker,
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

    pub(crate) fn vanish_assembly(&mut self, id: Assembly) -> Vec<Universe> {
        if let Some(index) = self.assembly.iter().position(|assembly| assembly == &id) {
            self.assembly.remove(index);
            vec![Universe::AssemblyVanished(id)]
        } else {
            vec![]
        }
    }

    pub(crate) fn vanish_door(&mut self, id: Door) -> Vec<Universe> {
        if let Some(index) = self.doors.iter().position(|door| door == &id) {
            self.doors.remove(index);
            vec![Universe::DoorVanished(id)]
        } else {
            vec![]
        }
    }

    pub(crate) fn vanish_crop(&mut self, id: Crop) -> Vec<Universe> {
        if let Some(index) = self.crops.iter().position(|crop| crop == &id) {
            self.crops.remove(index);
            vec![Universe::CropVanished(id)]
        } else {
            vec![]
        }
    }

    pub(crate) fn vanish_rest(&mut self, id: Rest) -> Vec<Universe> {
        if let Some(index) = self.rests.iter().position(|rest| rest == &id) {
            self.rests.remove(index);
            vec![Universe::RestVanished(id)]
        } else {
            vec![]
        }
    }

    pub(crate) fn vanish_cementer(&mut self, id: Cementer) -> Vec<Universe> {
        if let Some(index) = self.cementers.iter().position(|cementer| cementer == &id) {
            self.cementers.remove(index);
            vec![Universe::CementerVanished(id)]
        } else {
            vec![]
        }
    }

    pub(crate) fn vanish_composter(&mut self, id: Composter) -> Vec<Universe> {
        if let Some(index) = self
            .composters
            .iter()
            .position(|composter| composter == &id)
        {
            self.composters.remove(index);
            vec![Universe::ComposterVanished(id)]
        } else {
            vec![]
        }
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

    pub fn vanish_stack(&mut self, stack: Stack) -> Vec<Universe> {
        let index = self
            .stacks
            .iter()
            .position(|search| search.id == stack.id)
            .unwrap();
        self.stacks.remove(index);
        vec![Universe::StackVanished(stack)]
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

    pub fn change_activity(&mut self, farmer: Farmer, activity: Activity) -> Universe {
        self.farmers_activity.insert(farmer, activity);
        Universe::ActivityChanged { farmer, activity }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PlayerId(pub usize);

pub struct Player {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FarmerKey(pub usize);

pub struct FarmerKind {
    pub id: FarmerKey,
    pub name: String,
    pub body: BodyKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Activity {
    Idle,
    Usage,
    Surveying {
        equipment: Equipment,
        selection: usize,
    },
    Assembling {
        assembly: Assembly,
    },
    Resting {
        comfort: u8,
    },
}

impl Activity {
    pub fn as_assembling(&self) -> Result<Assembly, UniverseError> {
        match self {
            Activity::Assembling { assembly } => Ok(*assembly),
            _ => Err(UniverseError::ActivityMismatch),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ItemData {
    pub id: ItemId,
    pub kind: ItemKey,
    pub container: ContainerId,
    pub quantity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Farmer {
    pub id: usize,
    pub kind: FarmerKey,
    pub player: PlayerId,
    pub body: BodyId,
    pub hands: ContainerId,
    pub backpack: ContainerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TreeKey(pub usize);

pub struct TreeKind {
    pub id: TreeKey,
    pub name: String,
    pub barrier: Shared<BarrierKind>,
    pub plant: Shared<PlantKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Tree {
    pub id: usize,
    pub kind: TreeKey,
    pub plant: PlantId,
    pub barrier: BarrierId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FarmlandKey(pub usize);

pub struct FarmlandKind {
    pub id: FarmlandKey,
    pub name: String,
    pub space: Shared<SpaceKind>,
    pub soil: Shared<SoilKind>,
    pub grid: Shared<GridKind>,
    pub land: Shared<LandKind>,
    pub calendar: Shared<CalendarKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Farmland {
    pub id: usize,
    pub kind: FarmlandKey,
    pub space: SpaceId,
    pub soil: SoilId,
    pub grid: GridId,
    pub land: LandId,
    pub calendar: CalendarId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Construction {
    pub id: usize,
    pub container: ContainerId,
    pub grid: GridId,
    pub surveyor: SurveyorId,
    pub marker: Marker,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Deconstruction {
    pub id: usize,
    pub grid: GridId,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EquipmentKey(pub usize);

pub enum PurposeDescription {
    Surveying { surveyor: SurveyorKey },
    Moisture { sensor: usize },
}

pub struct EquipmentKind {
    pub id: EquipmentKey,
    pub name: String,
    pub purpose: PurposeDescription,
    pub barrier: Shared<BarrierKind>,
    pub item: Shared<ItemKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Purpose {
    Surveying { surveyor: SurveyorId },
    Moisture { sensor: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Equipment {
    pub id: usize,
    pub key: EquipmentKey,
    pub purpose: Purpose,
    pub barrier: BarrierId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Stack {
    pub id: usize,
    pub container: ContainerId,
    pub barrier: BarrierId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CropKey(pub usize);

pub struct CropKind {
    pub id: CropKey,
    pub name: String,
    pub plant: Shared<PlantKind>,
    pub barrier: Shared<BarrierKind>,
    pub sensor: Shared<SensorKind>,
    pub fruits: Shared<ItemKind>,
    pub residue: Shared<ItemKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Crop {
    pub id: usize,
    pub key: CropKey,
    pub plant: PlantId,
    pub barrier: BarrierId,
    pub sensor: SensorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CreatureKey(pub usize);

pub struct CreatureKind {
    pub id: CreatureKey,
    pub name: String,
    pub body: Shared<BodyKind>,
    pub animal: Shared<AnimalKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Creature {
    pub id: usize,
    pub key: CreatureKey,
    pub body: BodyId,
    pub animal: AnimalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AssemblyKey(pub usize);

pub enum AssemblyTarget {
    Door { door: Shared<DoorKind> },
    Cementer { cementer: Shared<CementerKind> },
    Composter { composter: Shared<ComposterKind> },
    Rest { rest: Shared<RestKind> },
}

pub struct AssemblyKind {
    pub key: AssemblyKey,
    pub name: String,
    pub target: AssemblyTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Assembly {
    pub id: usize,
    pub key: AssemblyKey,
    pub placement: PlacementId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DoorKey(pub usize);

pub struct DoorKind {
    pub key: DoorKey,
    pub name: String,
    pub barrier: Shared<BarrierKind>,
    pub kit: Shared<ItemKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Door {
    pub id: usize,
    pub key: DoorKey,
    pub barrier: BarrierId,
    pub placement: PlacementId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RestKey(pub usize);

pub struct RestKind {
    pub key: RestKey,
    pub name: String,
    pub comfort: u8,
    pub barrier: Shared<BarrierKind>,
    pub kit: Shared<ItemKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Rest {
    pub id: usize,
    pub key: RestKey,
    pub barrier: BarrierId,
    pub placement: PlacementId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CementerKey(pub usize);

pub struct CementerKind {
    pub key: CementerKey,
    pub name: String,
    pub kit: Shared<ItemKind>,
    pub barrier: Shared<BarrierKind>,
    pub device: Shared<DeviceKind>,
    pub input_offset: [i8; 2],
    pub input: Shared<ContainerKind>,
    pub output_offset: [i8; 2],
    pub output: Shared<ContainerKind>,
    pub cement: Shared<ItemKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Cementer {
    pub id: usize,
    pub key: CementerKey,
    pub input: ContainerId,
    pub device: DeviceId,
    pub output: ContainerId,
    pub barrier: BarrierId,
    pub placement: PlacementId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ComposterKey(pub usize);

pub struct ComposterKind {
    pub key: ComposterKey,
    pub name: String,
    pub kit: Shared<ItemKind>,
    pub barrier: Shared<BarrierKind>,
    pub device: Shared<DeviceKind>,
    pub input_offset: [i8; 2],
    pub input: Shared<ContainerKind>,
    pub output_offset: [i8; 2],
    pub output: Shared<ContainerKind>,
    pub compost: Shared<ItemKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Composter {
    pub id: usize,
    pub key: ComposterKey,
    pub input: ContainerId,
    pub device: DeviceId,
    pub output: ContainerId,
    pub barrier: BarrierId,
    pub placement: PlacementId,
}

// Models:
//
// Entity - aggregate of domain objects (hold identifiers)
// EntityPrefab - aggregate of domain object kinds (defines object properties)
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
