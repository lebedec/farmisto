use crate::transfer::{SyncReceiver, SyncSender};
use game::api::{GameResponse, LoginResult, PlayerRequest};
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Player {
    name: String,
    requests: Receiver<PlayerRequest>,
    responses: Sender<GameResponse>,
}

pub struct TrustedPlayerRequest {
    player: String,
    request: PlayerRequest,
}

pub struct Server {
    running: Arc<AtomicBool>,
    address: String,
    authorization: Receiver<Player>,
    players: HashMap<String, Player>,
}

pub struct Configuration {
    pub version: String,
    pub password: Option<String>,
}

impl Server {
    pub fn startup(config: Configuration) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let (listener_authorization, authorization) = channel();
        spawn_listener(running.clone(), config, listener_authorization);
        Self {
            running,
            address: detect_server_address(),
            authorization,
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
        todo!()
    }

    pub fn send(&mut self, player: String, response: GameResponse) {
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

    pub fn lost_players(&mut self) -> Vec<Player> {
        todo!()
    }

    pub fn terminate(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn spawn_listener(running: Arc<AtomicBool>, config: Configuration, authorization: Sender<Player>) {
    thread::spawn(move || {
        let address = "0.0.0.0:8080";
        info!(
            "Listen player connections on {:?} with {} version",
            address, config.version
        );
        let listener = match TcpListener::bind(address) {
            Ok(listener) => listener,
            Err(error) => {
                error!("Unable to bind listener, {:?}", error);
                return;
            }
        };
        let default_timeout = Some(Duration::from_secs(5));
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

                stream.set_read_timeout(default_timeout).unwrap();
                stream.set_write_timeout(default_timeout).unwrap();

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
                        if version != config.version {
                            warn!(
                                "Unable to authorize '{}' {}, version mismatch {} != {}",
                                player, peer, version, config.version
                            );
                            let result = GameResponse::Login {
                                result: LoginResult::VersionMismatch,
                            };
                            sender.send(&result).unwrap();
                            continue;
                        }
                        if password != config.password {
                            warn!(
                                "Unable to authorize '{}' {}, invalid password",
                                player, peer
                            );
                            let result = GameResponse::Login {
                                result: LoginResult::InvalidPassword,
                            };
                            sender.send(&result).unwrap();
                            continue;
                        }
                        player
                    }
                    _ => {
                        warn!("Unable to authorize {}, invalid {:?}", peer, request);
                        continue;
                    }
                };

                let (requests_sender, requests) = channel();
                let (responses, responses_receiver) = channel();

                let player = Player {
                    name: player,
                    requests,
                    responses,
                };

                let player_id = player.name.clone();
                thread::spawn(move || {
                    info!("Start player '{}' requests thread", player_id);
                    while let Some(request) = receiver.receive() {
                        if requests_sender.send(request).is_err() {
                            error!("Unable to receive request, server not working");
                            break;
                        }
                    }
                });

                let player_id = player.name.clone();
                thread::spawn(move || {
                    info!("Start player '{}' responses thread", player_id);

                    let result = GameResponse::Login {
                        result: LoginResult::Success,
                    };
                    sender.send(&result).unwrap();

                    for response in responses_receiver.iter() {
                        if sender.send(&response).is_none() {
                            error!("Unable to send response, connection lost");
                            break;
                        }
                    }
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
