use crate::transfer::{SyncReceiver, SyncSender};
use game::api::{GameResponse, LoginResult, PlayerRequest, API_VERSION};
use log::{error, info};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender, TryIter};
use std::thread;
use std::time::Duration;

pub struct Client {
    requests: Sender<PlayerRequest>,
    responses: Receiver<GameResponse>,
}

impl Client {
    pub fn connect(
        address: &str,
        player: String,
        password: Option<String>,
    ) -> Result<Self, String> {
        let address: SocketAddr = address.parse().unwrap();
        info!("Connect to {}, API version is {}", address, API_VERSION);
        let stream = TcpStream::connect(address).unwrap();
        let heartbeat = Duration::from_secs(2);
        let mut receiver = SyncReceiver {
            reader: stream.try_clone().unwrap(),
        };
        let mut sender = SyncSender {
            writer: stream.try_clone().unwrap(),
        };
        let authorization = PlayerRequest::Login {
            version: API_VERSION.to_string(),
            player,
            password,
        };
        sender.send(&authorization).unwrap();
        let response: Option<GameResponse> = receiver.receive();
        match response {
            Some(GameResponse::Login { result }) if result == LoginResult::Success => {
                info!("Authorization successful");
            }
            _ => return Err("Unable to connection, invalid response".to_string()),
        }

        let (requests, requests_receiver) = channel::<PlayerRequest>();
        let (responses_sender, responses) = channel::<GameResponse>();

        thread::spawn(move || {
            info!("Start client responses thread");
            while let Some(response) = receiver.receive() {
                if responses_sender.send(response).is_err() {
                    error!("Unable to receive response, client not working");
                    break;
                }
            }
            info!("Stop client responses thread");
        });

        thread::spawn(move || {
            info!("Start client requests thread");
            loop {
                let request = match requests_receiver.recv_timeout(heartbeat) {
                    Ok(request) => request,
                    Err(RecvTimeoutError::Timeout) => PlayerRequest::Heartbeat,
                    Err(RecvTimeoutError::Disconnected) => {
                        error!("Unable to send request, connection lost");
                        break;
                    }
                };
                if sender.send(&request).is_none() {
                    error!("Unable to send request, network error");
                    break;
                }
            }
            info!("Stop client requests thread");
        });

        let client = Client {
            requests,
            responses,
        };
        Ok(client)
    }

    pub fn is_connection_lost(&self) -> bool {
        todo!()
    }

    pub fn send(&self, request: PlayerRequest) {
        if self.requests.send(request).is_err() {
            error!("Unable to send request, client not working");
        }
    }

    #[inline]
    pub fn responses(&mut self) -> TryIter<GameResponse> {
        self.responses.try_iter()
    }

    pub fn disconnect(&mut self) {
        todo!()
    }
}
