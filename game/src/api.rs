use std::fmt::Debug;

use crate::assembling::{Assembling, AssemblingError, Rotation};
use crate::building::{Building, BuildingError, Marker, SurveyorId};
use crate::collections::DictionaryError;
use crate::inventory::{ContainerId, Inventory, InventoryError};
use crate::landscaping::{Landscaping, LandscapingError};
use crate::math::Tile;
use crate::model::{
    Activity, Cementer, Construction, Creature, Crop, Door, Equipment, Farmer, Stack, Universe,
    UniverseError,
};
use crate::physics::{Physics, PhysicsError};
use crate::planting::{Planting, PlantingError};
use crate::raising::{Raising, RaisingError};
use crate::working::{DeviceId, Working, WorkingError};

pub const API_VERSION: &str = "0.1";

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PlayerRequest {
    Heartbeat,
    Login {
        version: String,
        player: String,
        password: Option<String>,
    },
    Perform {
        action_id: usize,
        action: Action,
    },
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum GameResponse {
    Heartbeat,
    Events {
        events: Vec<Event>,
    },
    ActionError {
        action_id: usize,
        response: ActionResponse,
    },
    Login {
        result: LoginResult,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode, PartialEq)]
pub enum LoginResult {
    Success,
    VersionMismatch,
    InvalidPassword,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Action {
    EatCrop {
        creature: Creature,
        crop: Crop,
    },
    MoveCreature {
        creature: Creature,
        destination: [f32; 2],
    },
    TakeNap {
        creature: Creature,
    },
    Farmer {
        action: FarmerBound,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum FarmerBound {
    Move {
        destination: [f32; 2],
    },
    Install {
        tile: [usize; 2],
    },
    Uninstall {
        equipment: Equipment,
    },
    UseEquipment {
        equipment: Equipment,
    },
    CancelActivity,
    ToggleBackpack,
    ToggleSurveyingOption {
        option: u8,
    },
    Build {
        construction: Construction,
    },
    Survey {
        surveyor: SurveyorId,
        tile: [usize; 2],
        marker: Marker,
    },
    RemoveConstruction {
        construction: Construction,
    },
    TakeItemFromStack {
        stack: Stack,
    },
    TakeItemFromConstruction {
        construction: Construction,
    },
    TakeItemFromCementer {
        cementer: Cementer,
        container: ContainerId,
    },
    PutItemIntoStack {
        stack: Stack,
    },
    PutItemIntoConstruction {
        construction: Construction,
    },
    PutItemIntoCementer {
        cementer: Cementer,
        container: ContainerId,
    },
    DropItem {
        tile: [usize; 2],
    },
    PlantCrop {
        tile: [usize; 2],
    },
    WaterCrop {
        crop: Crop,
    },
    HarvestCrop {
        crop: Crop,
    },
    StartAssembly {
        pivot: [usize; 2],
        rotation: Rotation,
    },
    MoveAssembly {
        pivot: [usize; 2],
        rotation: Rotation,
    },
    FinishAssembly {
        pivot: [usize; 2],
        rotation: Rotation,
    },
    CancelAssembly,
    ToggleDoor {
        door: Door,
    },
    DisassembleDoor {
        door: Door,
    },
    DisassembleCementer {
        cementer: Cementer,
    },
    RepairDevice {
        device: DeviceId,
    },
    ToggleDevice {
        device: DeviceId,
    },
    PlowFarmland {
        place: Tile,
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct ActionResponse {
    pub error: ActionError,
    pub farmer: Farmer,
    pub correction: Activity,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum ActionError {
    Building(BuildingError),
    Inventory(InventoryError),
    Universe(UniverseError),
    Physics(PhysicsError),
    Planting(PlantingError),
    Raising(RaisingError),
    Assembling(AssemblingError),
    Working(WorkingError),
    Landscaping(LandscapingError),

    Inconsistency(DictionaryError),

    PlayerFarmerNotFound(String),
    FarmerBodyNotFound(Farmer),
    ConstructionContainerNotFound(Construction),
    ConstructionContainerNotInitialized(Construction),
    ConstructionContainsUnexpectedItem(Construction),

    TargetUnreachable,

    Test,
}

impl From<DictionaryError> for ActionError {
    fn from(error: DictionaryError) -> Self {
        Self::Inconsistency(error)
    }
}

impl From<BuildingError> for ActionError {
    fn from(error: BuildingError) -> Self {
        Self::Building(error)
    }
}

impl From<InventoryError> for ActionError {
    fn from(error: InventoryError) -> Self {
        Self::Inventory(error)
    }
}

impl From<UniverseError> for ActionError {
    fn from(error: UniverseError) -> Self {
        Self::Universe(error)
    }
}

impl From<PhysicsError> for ActionError {
    fn from(error: PhysicsError) -> Self {
        Self::Physics(error)
    }
}

impl From<PlantingError> for ActionError {
    fn from(error: PlantingError) -> Self {
        Self::Planting(error)
    }
}

impl From<RaisingError> for ActionError {
    fn from(error: RaisingError) -> Self {
        Self::Raising(error)
    }
}

impl From<AssemblingError> for ActionError {
    fn from(error: AssemblingError) -> Self {
        Self::Assembling(error)
    }
}

impl From<WorkingError> for ActionError {
    fn from(error: WorkingError) -> Self {
        Self::Working(error)
    }
}

impl From<LandscapingError> for ActionError {
    fn from(error: LandscapingError) -> Self {
        Self::Landscaping(error)
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Event {
    UniverseStream(Vec<Universe>),
    PhysicsStream(Vec<Physics>),
    BuildingStream(Vec<Building>),
    InventoryStream(Vec<Inventory>),
    PlantingStream(Vec<Planting>),
    RaisingStream(Vec<Raising>),
    AssemblingStream(Vec<Assembling>),
    WorkingStream(Vec<Working>),
    LandscapingStream(Vec<Landscaping>),
}

impl Into<Event> for Vec<Universe> {
    fn into(self) -> Event {
        Event::UniverseStream(self)
    }
}

impl Into<Event> for Vec<Physics> {
    fn into(self) -> Event {
        Event::PhysicsStream(self)
    }
}

impl Into<Event> for Vec<Building> {
    fn into(self) -> Event {
        Event::BuildingStream(self)
    }
}

impl Into<Event> for Vec<Inventory> {
    fn into(self) -> Event {
        Event::InventoryStream(self)
    }
}

impl Into<Event> for Vec<Planting> {
    fn into(self) -> Event {
        Event::PlantingStream(self)
    }
}

impl Into<Event> for Vec<Raising> {
    fn into(self) -> Event {
        Event::RaisingStream(self)
    }
}

impl Into<Event> for Vec<Assembling> {
    fn into(self) -> Event {
        Event::AssemblingStream(self)
    }
}

impl Into<Event> for Vec<Working> {
    fn into(self) -> Event {
        Event::WorkingStream(self)
    }
}

impl Into<Event> for Vec<Landscaping> {
    fn into(self) -> Event {
        Event::LandscapingStream(self)
    }
}

impl Into<Event> for Planting {
    fn into(self) -> Event {
        Event::PlantingStream(vec![self])
    }
}

impl Into<Event> for Universe {
    fn into(self) -> Event {
        Event::UniverseStream(vec![self])
    }
}

impl Into<Event> for Physics {
    fn into(self) -> Event {
        Event::PhysicsStream(vec![self])
    }
}

impl Into<Event> for Building {
    fn into(self) -> Event {
        Event::BuildingStream(vec![self])
    }
}

impl Into<Event> for Inventory {
    fn into(self) -> Event {
        Event::InventoryStream(vec![self])
    }
}

impl Into<Event> for Raising {
    fn into(self) -> Event {
        Event::RaisingStream(vec![self])
    }
}

impl Into<Event> for Assembling {
    fn into(self) -> Event {
        Event::AssemblingStream(vec![self])
    }
}

impl Into<Event> for Working {
    fn into(self) -> Event {
        Event::WorkingStream(vec![self])
    }
}

impl Into<Event> for Landscaping {
    fn into(self) -> Event {
        Event::LandscapingStream(vec![self])
    }
}
