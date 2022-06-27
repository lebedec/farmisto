use game::api::{GameResponse, PlayerRequest};
use log::error;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Connector {}

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub fn connect(address: &str, player: String, password: Option<String>) {
        todo!()
    }
}

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct GameResponseReceiver {
    pub reader: TcpStream,
}

impl GameResponseReceiver {
    pub fn receive(&mut self) -> Option<GameResponse> {
        let mut buffer = [0_u8; 2];
        if let Err(error) = self.reader.read_exact(&mut buffer) {
            error!("Unable to receive because of header read, {}", error);
            return None;
        }
        let length = u16::from_be_bytes(buffer) as usize;

        let mut buffer = vec![0; length];
        if let Err(error) = self.reader.read_exact(buffer.as_mut_slice()) {
            error!("Unable to receive because of body read, {}", error);
            return None;
        }
        match GameResponse::from_bytes(buffer.as_slice()) {
            Ok(response) => Some(response),
            Err(error) => {
                error!("Unable to receive because of deserialization, {}", error);
                None
            }
        }
    }
}

pub struct PlayerRequestSender {
    pub writer: TcpStream,
}

impl PlayerRequestSender {
    pub fn send(&mut self, request: PlayerRequest) -> Option<()> {
        match request.as_bytes() {
            Ok(body) => {
                let header = match u16::try_from(body.len()) {
                    Ok(body_length) => body_length.to_be_bytes(),
                    Err(error) => {
                        error!("Unable to send because of body length {}", error);
                        return None;
                    }
                };
                if let Err(error) = self.writer.write_all(&header) {
                    error!("Unable to send because of header write, {}", error);
                    return None;
                }
                if let Err(error) = self.writer.write_all(&body) {
                    error!("Unable to send because of body write, {}", error);
                    return None;
                }
            }
            Err(error) => {
                error!("Unable to send because of serialization, {}", error)
            }
        }

        return Some(());
    }
}
