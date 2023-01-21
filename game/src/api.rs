use crate::building::{Building, BuildingError, Room};
use crate::inventory::{Inventory, InventoryError};
use crate::model::{
    Construction, Farmer, FarmerId, FarmerKey, FarmlandId, FarmlandKey, TreeId, TreeKey,
    UniverseError,
};
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
        error: ActionError,
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
    DoSomething,
    DoAnything { id: usize, position: [f32; 2] },
    MoveFarmer { destination: [f32; 2] },
    BuildWall { cell: [usize; 2] },
    Construct { construction: Construction },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum ActionError {
    Building(BuildingError),
    Inventory(InventoryError),
    Universe(UniverseError),
    PlayerFarmerNotFound(String),
    FarmerBodyNotFound(FarmerId),
    ConstructionContainerNotFound(Construction),
    ConstructionContainerNotInitialized(Construction),
    ConstructionContainsUnexpectedItem(Construction),
    Boba,
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

#[derive(bincode::Encode, bincode::Decode)]
pub enum Event {
    BarrierHintAppeared {
        id: usize,
        kind: usize,
        position: [f32; 2],
        bounds: [f32; 2],
    },
    TreeAppeared {
        id: TreeId,
        kind: TreeKey,
        position: [f32; 2],
        growth: f32,
    },
    TreeUpdated {
        id: TreeId,
    },
    TreeVanished(TreeId),
    FarmlandAppeared {
        id: FarmlandId,
        kind: FarmlandKey,
        map: Vec<Vec<[f32; 2]>>,
        platform: Vec<Vec<u32>>,
        platform_shapes: Vec<Room>,
    },
    FarmlandPlatformUpdated {
        id: FarmlandId,
        platform: Vec<Vec<u32>>,
        platform_shapes: Vec<Room>,
    },
    FarmlandUpdated {
        id: FarmlandId,
        map: Vec<Vec<[f32; 2]>>,
    },
    FarmlandVanished(FarmlandId),
    FarmerAppeared {
        id: FarmerId,
        kind: FarmerKey,
        player: String,
        position: [f32; 2],
    },
    FarmerVanished(FarmerId),
    FarmerMoved {
        id: FarmerId,
        position: [f32; 2],
    },
    Building(Building),
    Inventory(Inventory),
}

impl From<Building> for Event {
    fn from(event: Building) -> Self {
        Self::Building(event)
    }
}

impl From<Inventory> for Event {
    fn from(event: Inventory) -> Self {
        Self::Inventory(event)
    }
}

trait MyInto {
    fn into(self) -> Vec<Event>;
}

impl<T> MyInto for Vec<T>
where
    T: Into<Event>,
{
    fn into(self) -> Vec<Event> {
        occur(self)
    }
}

pub fn occur<T>(events: Vec<T>) -> Vec<Event>
where
    T: Into<Event>,
{
    events.into_iter().map(|event| event.into()).collect()
}
