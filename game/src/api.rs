use crate::building::Shape;
use crate::model::{FarmerId, FarmerKey, FarmlandId, FarmlandKey, TreeId, TreeKey};
use crate::planting::Cell;
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
    Events { events: Vec<Event> },
    Login { result: LoginResult },
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
        map: Vec<Vec<Cell>>,
        platform: Vec<Vec<u32>>,
        platform_shapes: Vec<Shape>,
    },
    FarmlandPlatformUpdated {
        id: FarmlandId,
        platform: Vec<Vec<u32>>,
        platform_shapes: Vec<Shape>,
    },
    FarmlandUpdated {
        id: FarmlandId,
        map: Vec<Vec<Cell>>,
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
}
