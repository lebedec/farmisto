use crate::transfer::{SyncReceiver, SyncSender};
use game::api::{GameResponse, LoginResult, PlayerRequest, API_VERSION};
use lazy_static::lazy_static;
use log::{error, info};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender, TryIter};
use std::thread;
use std::time::{Duration, Instant};

lazy_static! {
    static ref METRIC_SENT_BYTES: prometheus::IntCounter =
        prometheus::register_int_counter!("client_sent_bytes", "client_sent_bytes").unwrap();
    static ref METRIC_RECEIVED_BYTES: prometheus::IntCounter =
        prometheus::register_int_counter!("client_received_bytes", "client_received_bytes")
            .unwrap();
}

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
        let client_connect = move || {
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
            let response: Option<(_, GameResponse)> = receiver.receive();
            match response {
                Some((_, GameResponse::Login { result })) if result == LoginResult::Success => {
                    info!("Authorization successful");
                }
                _ => {
                    error!("Unable to connection, invalid response");
                    return;
                }
            }
            let client_responses = move || {
                info!("Start client responses thread");
                'running: loop {
                    let mut responses = vec![];
                    let time = Instant::now();
                    loop {
                        match receiver.receive() {
                            Some((bytes, response)) => {
                                METRIC_RECEIVED_BYTES.inc_by(bytes as u64);
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
                            break 'running;
                        }
                    }
                }
                info!("Stop client responses thread");
            };
            thread::Builder::new()
                .name("client_responses".into())
                .spawn(client_responses)
                .unwrap();

            let client_requests = move || {
                info!("Start client requests thread");
                'thread: loop {
                    let mut requests = vec![];
                    let time = Instant::now();
                    loop {
                        let request = match requests_receiver.recv_timeout(heartbeat) {
                            Ok(request) => request,
                            Err(RecvTimeoutError::Timeout) => PlayerRequest::Heartbeat,
                            Err(RecvTimeoutError::Disconnected) => {
                                error!("Unable to send request, connection lost");
                                break 'thread;
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
                        match sender.send(&request) {
                            Some(bytes) => {
                                METRIC_SENT_BYTES.inc_by(bytes as u64);
                            }
                            None => {
                                error!("Unable to send request, network error");
                                break;
                            }
                        }
                    }
                }
                info!("Stop client requests thread");
            };
            thread::Builder::new()
                .name("client_requests".into())
                .spawn(client_requests)
                .unwrap();
        };
        thread::Builder::new()
            .name("client_connect".into())
            .spawn(client_connect)
            .unwrap();

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
