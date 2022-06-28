use bincode::error::{DecodeError, EncodeError};
use bincode::{Decode, Encode};
use log::{error, info};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct SyncReceiver {
    pub reader: TcpStream,
}

impl SyncReceiver {
    pub fn receive<T: Decode>(&mut self) -> Option<T> {
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
        match decode(buffer.as_slice()) {
            Ok(response) => Some(response),
            Err(error) => {
                error!("Unable to receive because of deserialization, {}", error);
                None
            }
        }
    }
}

pub struct SyncSender {
    pub writer: TcpStream,
}

impl SyncSender {
    pub fn send_body(&mut self, body: Vec<u8>) -> Option<()> {
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
            None
        } else {
            if body.len() > 100 {
                info!("Sent {} bytes body", body.len());
            }
            Some(())
        }
    }

    pub fn send<T: Encode>(&mut self, value: &T) -> Option<()> {
        match encode(value) {
            Ok(body) => self.send_body(body),
            Err(error) => {
                error!("Unable to send because of serialization, {}", error);
                None
            }
        }
    }
}

#[inline]
pub fn decode<T: Decode>(data: &[u8]) -> Result<T, DecodeError> {
    let config = bincode::config::standard();
    let (response, _) = bincode::decode_from_slice(data, config)?;
    Ok(response)
}

#[inline]
pub fn encode<T: Encode>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let config = bincode::config::standard();
    bincode::encode_to_vec(value, config)
}
