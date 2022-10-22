use crate::transfer::{encode, SyncReceiver, SyncSender};
use game::api::{GameResponse, LoginResult, PlayerRequest, API_VERSION};
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Player {
    name: String,
    requests: Receiver<PlayerRequest>,
    responses: Sender<Vec<u8>>,
}

pub struct TrustedPlayerRequest {
    pub player: String,
    pub request: PlayerRequest,
}

pub struct TcpServer {
    running: Arc<AtomicBool>,
    address: String,
    authorization: Receiver<Player>,
    disconnection: Receiver<String>,
    players: HashMap<String, Player>,
}

pub struct Configuration {
    pub host: String,
    pub port: u32,
    pub password: Option<String>,
}

impl TcpServer {
    pub fn startup(config: Configuration) -> Self {
        let address = detect_server_address();
        info!("Start server {}", address);
        let running = Arc::new(AtomicBool::new(true));
        let (listener_authorization, authorization) = channel();
        let (listener_disconnection, disconnection) = channel();
        spawn_listener(
            running.clone(),
            config,
            listener_authorization,
            listener_disconnection,
        );
        Self {
            running,
            address,
            authorization,
            disconnection,
            players: HashMap::new(),
        }
    }

    #[inline]
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn accept_players(&mut self) -> Vec<String> {
        let mut players = vec![];
        for player in self.authorization.try_iter() {
            let name = player.name.clone();
            players.push(name.clone());
            self.players.insert(name, player);
        }
        players
    }

    pub fn lost_players(&mut self) -> Vec<String> {
        let mut players = vec![];
        for player in self.disconnection.try_iter() {
            self.players.remove(&player);
            players.push(player);
        }
        players
    }

    pub fn requests(&mut self) -> Vec<TrustedPlayerRequest> {
        let mut requests = vec![];
        for player in self.players.values() {
            let player_requests = player
                .requests
                .try_iter()
                .map(|request| TrustedPlayerRequest {
                    player: player.name.clone(),
                    request,
                });
            requests.extend(player_requests);
        }
        requests
    }

    pub fn broadcast(&mut self, response: GameResponse) {
        // first of all encode response only once
        // todo: move encoding to separated thread
        // todo: zero copy senders
        let response = encode(&response).unwrap();
        for player in self.players.values() {
            if player.responses.send(response.clone()).is_err() {
                error!(
                    "Unable to send response, player '{}' connection lost",
                    player.name
                );
                continue;
            }
        }
    }

    pub fn send(&mut self, player: String, response: GameResponse) {
        let response = encode(&response).unwrap();
        match self.players.get_mut(&player) {
            Some(player) => {
                if player.responses.send(response).is_err() {
                    error!(
                        "Unable to send response, player '{}' connection lost",
                        player.name
                    );
                }
            }
            None => {
                error!("Unable to send response, player '{}' not found", player);
            }
        }
    }

    pub fn terminate(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn spawn_listener(
    running: Arc<AtomicBool>,
    config: Configuration,
    authorization: Sender<Player>,
    disconnection: Sender<String>,
) {
    thread::spawn(move || {
        let address = format!("0.0.0.0:{}", config.port);
        info!(
            "Listen player connections on {:?}, API version is {}",
            address, API_VERSION
        );
        let listener = match TcpListener::bind(address) {
            Ok(listener) => listener,
            Err(error) => {
                error!("Unable to bind listener, {:?}", error);
                return;
            }
        };

        let default_timeout = Duration::from_secs(5);
        let heartbeat = Duration::from_secs(2);

        while running.load(Ordering::Relaxed) {
            // todo: nonblocking
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(stream) => stream,
                    Err(error) => {
                        error!("Unable to establish connection, {:?}", error);
                        continue;
                    }
                };
                let peer = stream.peer_addr().unwrap().to_string();
                info!("New connection from {:?}", peer);

                stream.set_read_timeout(Some(default_timeout)).unwrap();
                stream.set_write_timeout(Some(default_timeout)).unwrap();

                let mut receiver = SyncReceiver {
                    reader: stream.try_clone().unwrap(),
                };
                let mut sender = SyncSender {
                    writer: stream.try_clone().unwrap(),
                };

                // authorization blocks new player connections,
                // (should be super fast)
                let request: Option<PlayerRequest> = receiver.receive();
                let player = match request {
                    Some(PlayerRequest::Login {
                        version,
                        player,
                        password,
                    }) => {
                        if version != API_VERSION {
                            warn!(
                                "Unable to authorize '{}' {}, version mismatch {} != {}",
                                player, peer, version, API_VERSION
                            );
                            let result = LoginResult::VersionMismatch;
                            let response = GameResponse::Login { result };
                            sender.send(&response).unwrap();
                            continue;
                        }

                        if password != config.password {
                            warn!(
                                "Unable to authorize '{}' {}, invalid password",
                                player, peer
                            );
                            let result = LoginResult::InvalidPassword;
                            let response = GameResponse::Login { result };
                            sender.send(&response).unwrap();
                            continue;
                        }

                        player
                    }
                    _ => {
                        warn!("Unable to authorize {}, invalid {:?}", peer, request);
                        continue;
                    }
                };

                info!("Authorization of '{}' player successful", player);

                let (requests_sender, requests) = channel();
                let (responses, responses_receiver) = channel();

                let player = Player {
                    name: player,
                    requests,
                    responses,
                };

                let player_id = player.name.clone();
                let player_disconnection = disconnection.clone();
                thread::spawn(move || {
                    info!("Start player '{}' requests thread", player_id);
                    while let Some(request) = receiver.receive() {
                        if requests_sender.send(request).is_err() {
                            error!("Unable to receive request, server not working");
                            break;
                        }
                    }
                    info!("Stop player '{}' requests thread", player_id);

                    if player_disconnection.send(player_id).is_err() {
                        error!("Unable to disconnect player, server not working")
                    }
                });

                let player_id = player.name.clone();
                thread::spawn(move || {
                    info!("Start player '{}' responses thread", player_id);

                    let result = LoginResult::Success;
                    let response = GameResponse::Login { result };
                    sender.send(&response).unwrap();

                    loop {
                        let response = match responses_receiver.recv_timeout(heartbeat) {
                            Ok(response) => response,
                            Err(RecvTimeoutError::Timeout) => {
                                encode(&GameResponse::Heartbeat).unwrap()
                            }
                            Err(RecvTimeoutError::Disconnected) => {
                                error!("Unable to send response, connection lost");
                                break;
                            }
                        };
                        if sender.send_body(response).is_none() {
                            error!("Unable to send response, network error");
                            break;
                        }
                    }
                    info!("Stop player '{}' responses thread", player_id);
                });

                if authorization.send(player).is_err() {
                    error!("Unable to authorize {}, server not working", peer);
                    break;
                }
            }
        }
        info!("Server listener terminated")
    });
}

#[cfg(unix)]
fn detect_server_address() -> String {
    match Command::new("sh")
        .arg("-c")
        .arg("ifconfig | grep 'inet ' | grep -v 127.0.0.1 | cut -d' ' -f2")
        .output()
        .map_err(|err| err.to_string())
        .and_then(|output| String::from_utf8(output.stdout).map_err(|err| err.to_string()))
    {
        Ok(ip) => ip.trim().to_string(),
        Err(error) => {
            error!("Unable to detect server local IP, {}", error);
            "127.0.0.1".to_string()
        }
    }
}

#[cfg(windows)]
fn detect_server_address() -> String {
    match Command::new("ipconfig")
        //.args(["interface", "ip", "show", "config"])
        .output()
        .map_err(|err| err.to_string())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
    {
        Ok(configuration) => {
            let mut ip = "127.0.0.1".to_string();
            for line in configuration.split("\n") {
                let line = line.trim();
                if line.starts_with("IPv4") {
                    ip = line.split_whitespace().last().unwrap().to_string();
                    break;
                }
            }
            ip
        }
        Err(error) => {
            error!("Unable to detect server local IP, {}", error);
            "127.0.0.1".to_string()
        }
    }
}
