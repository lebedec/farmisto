use crate::model::{TreeId, TreeKey};
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
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum Event {
    TreeAppeared {
        id: TreeId,
        kind: TreeKey,
        position: [f32; 2],
        growth: f32,
    },
    TreeVanished {
        id: TreeId,
    },
}
