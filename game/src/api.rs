use crate::building::{Building, BuildingError};
use crate::inventory::{Inventory, InventoryError};
use crate::model::{Construction, Drop, Farmer, Tile, Universe, UniverseError};
use crate::physics::{Physics, PhysicsError};
use crate::planting::Planting;
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
    Survey { target: Tile },
    TakeItem { drop: Drop },
    DropItem { tile: [usize; 2] },
    PutItem { drop: Drop },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum ActionError {
    Building(BuildingError),
    Inventory(InventoryError),
    Universe(UniverseError),
    Physics(PhysicsError),
    PlayerFarmerNotFound(String),
    FarmerBodyNotFound(Farmer),
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

impl From<PhysicsError> for ActionError {
    fn from(error: PhysicsError) -> Self {
        Self::Physics(error)
    }
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum Event {
    Universe(Vec<Universe>),
    Physics(Vec<Physics>),
    Building(Vec<Building>),
    Inventory(Vec<Inventory>),
    Planting(Vec<Planting>),
}
