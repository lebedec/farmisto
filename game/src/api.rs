use crate::building::{Building, BuildingError, Marker, Stake, Structure, SurveyorId};
use crate::inventory::{Inventory, InventoryError, ItemId};
use crate::model::{
    Activity, Construction, Creature, Crop, Equipment, EquipmentKey, Farmer, Stack, Universe,
    UniverseError,
};
use crate::physics::{Physics, PhysicsError};
use crate::planting::{Planting, PlantingError};
use crate::raising::{Raising, RaisingError};
use std::fmt::Debug;

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
    ToggleSurveyingOption,
    TakeMaterial {
        construction: Construction,
    },
    Build {
        construction: Construction,
    },
    PutMaterial {
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
    TakeItem {
        stack: Stack,
    },
    DropItem {
        tile: [usize; 2],
    },
    PutItem {
        stack: Stack,
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
    PlayerFarmerNotFound(String),
    FarmerBodyNotFound(Farmer),
    ConstructionContainerNotFound(Construction),
    ConstructionContainerNotInitialized(Construction),
    ConstructionContainsUnexpectedItem(Construction),
    ItemHasNoEquipmentFunction,
    EquipmentKindNotFound { key: EquipmentKey },
    Test,
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

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Event {
    Universe(Vec<Universe>),
    Physics(Vec<Physics>),
    Building(Vec<Building>),
    Inventory(Vec<Inventory>),
    Planting(Vec<Planting>),
    Raising(Vec<Raising>),
}

impl Into<Event> for Vec<Universe> {
    fn into(self) -> Event {
        Event::Universe(self)
    }
}

impl Into<Event> for Vec<Physics> {
    fn into(self) -> Event {
        Event::Physics(self)
    }
}

impl Into<Event> for Vec<Building> {
    fn into(self) -> Event {
        Event::Building(self)
    }
}

impl Into<Event> for Vec<Inventory> {
    fn into(self) -> Event {
        Event::Inventory(self)
    }
}

impl Into<Event> for Vec<Planting> {
    fn into(self) -> Event {
        Event::Planting(self)
    }
}

impl Into<Event> for Vec<Raising> {
    fn into(self) -> Event {
        Event::Raising(self)
    }
}

impl Into<Event> for Planting {
    fn into(self) -> Event {
        Event::Planting(vec![self])
    }
}

impl Into<Event> for Universe {
    fn into(self) -> Event {
        Event::Universe(vec![self])
    }
}

impl Into<Event> for Physics {
    fn into(self) -> Event {
        Event::Physics(vec![self])
    }
}

impl Into<Event> for Building {
    fn into(self) -> Event {
        Event::Building(vec![self])
    }
}

impl Into<Event> for Inventory {
    fn into(self) -> Event {
        Event::Inventory(vec![self])
    }
}

impl Into<Event> for Raising {
    fn into(self) -> Event {
        Event::Raising(vec![self])
    }
}
