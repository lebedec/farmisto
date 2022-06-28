use crate::physics::Physics;
use crate::shapes::Shapes;

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

#[derive(Debug, bincode::Encode, bincode::Decode)]
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

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Event {
    ShapesEvents(Shapes),
    PhysicsEvents(Physics),
    EntityAppeared { id: usize, kind: usize },
    EntityVanished { id: usize },
    EventB,
}
