use game::api::{GameResponse, PlayerRequest};

struct Client {}

impl Client {
    pub fn connect(address: &str, player: String, password: Option<String>) -> Self {
        todo!()
    }

    pub fn is_connection_lost(&self) -> bool {
        todo!()
    }

    pub fn send(&self, request: PlayerRequest) {
        todo!()
    }

    pub fn responses(&mut self) -> Vec<GameResponse> {
        todo!()
    }

    pub fn disconnect(&mut self) {
        todo!()
    }
}
