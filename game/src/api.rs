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

impl PlayerRequest {
    pub fn as_bytes(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(self, config)
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum GameResponse {
    Pong,
    Events { events: Vec<Event> },
}

impl GameResponse {
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<GameResponse, bincode::error::DecodeError> {
        let config = bincode::config::standard();
        let (response, _) = bincode::decode_from_slice(data, config)?;
        Ok(response)
    }
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
