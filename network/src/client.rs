use crate::transfer::{SyncReceiver, SyncSender};
use game::api::{GameResponse, LoginResult, PlayerRequest, API_VERSION};
use log::{error, info};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender, TryIter};
use std::thread;
use std::time::{Duration, Instant};

pub struct TcpClient {
    pub player: String,
    requests: Sender<PlayerRequest>,
    responses: Receiver<GameResponse>,
}

impl TcpClient {
    pub fn connect(
        address: &str,
        player: String,
        password: Option<String>,
    ) -> Result<Self, String> {
        let address: SocketAddr = address.parse().unwrap();
        info!("Connect to {}, API version is {}", address, API_VERSION);

        let (requests, requests_receiver) = channel::<PlayerRequest>();
        let (responses_sender, responses) = channel::<GameResponse>();

        let thread_player = player.clone();
        thread::spawn(move || {
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
                player: thread_player,
                password,
            };
            sender.send(&authorization).unwrap();
            let response: Option<GameResponse> = receiver.receive();
            match response {
                Some(GameResponse::Login { result }) if result == LoginResult::Success => {
                    info!("Authorization successful");
                }
                _ => {
                    error!("Unable to connection, invalid response");
                    return;
                }
            }
            thread::spawn(move || {
                info!("Start client responses thread");
                loop {
                    let mut responses = vec![];
                    let time = Instant::now();
                    loop {
                        match receiver.receive() {
                            Some(response) => {
                                responses.push(response);
                            }
                            None => {
                                error!("Unable to receive response, server not working");
                                break;
                            }
                        }
                        // - buffering
                        // - downstream latency simulation
                        if time.elapsed() > Duration::from_millis(0) {
                            break;
                        }
                    }
                    for response in responses {
                        if responses_sender.send(response).is_err() {
                            error!("Unable to receive response, client not working");
                            break;
                        }
                    }
                }
                info!("Stop client responses thread");
            });

            thread::spawn(move || {
                info!("Start client requests thread");
                loop {
                    let mut requests = vec![];
                    let time = Instant::now();
                    loop {
                        let request = match requests_receiver.recv_timeout(heartbeat) {
                            Ok(request) => request,
                            Err(RecvTimeoutError::Timeout) => PlayerRequest::Heartbeat,
                            Err(RecvTimeoutError::Disconnected) => {
                                error!("Unable to send request, connection lost");
                                break;
                            }
                        };
                        requests.push(request);

                        // - buffering
                        // - upstream latency simulation
                        if time.elapsed() > Duration::from_millis(0) {
                            break;
                        }
                    }
                    for request in requests {
                        if sender.send(&request).is_none() {
                            error!("Unable to send request, network error");
                            break;
                        }
                    }
                }
                info!("Stop client requests thread");
            });
        });

        let client = TcpClient {
            player,
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
