use std::io::{Read, Write};
use std::net::TcpStream;

use bincode::error::{DecodeError, EncodeError};

use log::error;

pub struct SyncReceiver {
    pub reader: TcpStream,
}

const HEADER_LENGTH: usize = 4;

impl SyncReceiver {
    pub fn receive<T: serde::de::DeserializeOwned>(&mut self) -> Option<(usize, T)> {
        let mut buffer = [0_u8; HEADER_LENGTH];
        if let Err(error) = self.reader.read_exact(&mut buffer) {
            error!("Unable to receive because of header read, {}", error);
            return None;
        }
        let length = u32::from_be_bytes(buffer) as usize;
        let mut buffer = vec![0; length];
        if let Err(error) = self.reader.read_exact(buffer.as_mut_slice()) {
            error!("Unable to receive because of body read, {}", error);
            return None;
        }

        // let mut rdr = snap::read::FrameDecoder::new(Cursor::new(buffer));
        // let mut out = vec![];
        // rdr.read_to_end(&mut out).unwrap();
        // let buffer = out;

        match decode(&buffer) {
            Ok(response) => Some((HEADER_LENGTH + length, response)),
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
    pub fn send_body(&mut self, body: Vec<u8>) -> Option<usize> {
        // let mut buf = vec![];
        // {
        //     let mut wtr = snap::write::FrameEncoder::new(&mut buf);
        //     wtr.write_all(&body).unwrap();
        // }
        // let body = buf;

        let header = match u32::try_from(body.len()) {
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
            Some(HEADER_LENGTH + body.len())
        }
    }

    pub fn send<T: serde::Serialize>(&mut self, value: &T) -> Option<usize> {
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
pub fn decode<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, DecodeError> {
    let config = bincode::config::standard();
    let (response, _) = bincode::serde::decode_from_slice(data, config)?;
    Ok(response)
}

#[inline]
pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let config = bincode::config::standard();
    // bincode::encode_to_vec(value, config)
    bincode::serde::encode_to_vec(value, config)
}
