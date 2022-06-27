#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PlayerRequest {
    Ping,
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
    Pong,
    Events { events: Vec<Event> },
    Login { result: LoginResult },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
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
    EventA,
    EventB,
}
