use crate::transfer::{encode, SyncReceiver, SyncSender};
use game::api::{Event, GameResponse, LoginResult, PlayerRequest, API_VERSION};
use lazy_static::lazy_static;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

lazy_static! {
    static ref SERVER_SENT_BYTES: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!(
            "server_sent_bytes",
            "server_sent_bytes",
            &["player"]
        )
        .unwrap();
    static ref SERVER_SENT_RESPONSES_TOTAL: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!(
            "server_sent_responses_total",
            "server_sent_responses_total",
            &["player"]
        )
        .unwrap();
    static ref SERVER_RTT_SECONDS: prometheus::GaugeVec =
        prometheus::register_gauge_vec!("server_rtt_seconds", "server_rtt_seconds", &["player"])
            .unwrap();
    static ref SERVER_SENT_EVENTS_TOTAL: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!(
            "server_sent_events_total",
            "server_sent_events_total",
            &["domain"]
        )
        .unwrap();
    static ref SERVER_SENT_EVENT_STREAMS_TOTAL: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!(
            "server_sent_event_streams_total",
            "server_sent_event_streams_total",
            &["domain"]
        )
        .unwrap();
    static ref SERVER_RECEIVED_BYTES: prometheus::IntCounter =
        prometheus::register_int_counter!("server_received_bytes", "server_received_bytes")
            .unwrap();
    static ref SERVER_RECEIVED_REQUESTS_TOTAL: prometheus::IntCounter =
        prometheus::register_int_counter!(
            "server_received_requests_total",
            "server_received_requests_total"
        )
        .unwrap();
}

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
    pub save_file: String,
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

    pub fn has_player(&self, player: &str) -> bool {
        self.players.contains_key(player)
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
        let response_data = encode(&response).unwrap();
        for player in self.players.values() {
            self.measure_response(&response);
            if player.responses.send(response_data.clone()).is_err() {
                error!(
                    "Unable to send response, player '{}' connection lost",
                    player.name
                );
                continue;
            }
        }
    }

    pub fn send(&mut self, player: String, response: GameResponse) {
        self.measure_response(&response);
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

    fn measure_response(&self, response: &GameResponse) {
        match response {
            GameResponse::Heartbeat { .. } => {}
            GameResponse::Trip { .. } => {}
            GameResponse::Events { events } => {
                for event in events {
                    match event {
                        Event::TimingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["timing"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["timing"])
                                .inc();
                        }
                        Event::UniverseStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["universe"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["universe"])
                                .inc();
                        }
                        Event::PhysicsStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["physics"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["physics"])
                                .inc();
                        }
                        Event::BuildingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["building"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["building"])
                                .inc();
                        }
                        Event::InventoryStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["inventory"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["inventory"])
                                .inc();
                        }
                        Event::PlantingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["planting"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["planting"])
                                .inc();
                        }
                        Event::RaisingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["raising"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["raising"])
                                .inc();
                        }
                        Event::AssemblingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["assembling"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["assembling"])
                                .inc();
                        }
                        Event::WorkingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["working"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["working"])
                                .inc();
                        }
                        Event::LandscapingStream(events) => {
                            SERVER_SENT_EVENTS_TOTAL
                                .with_label_values(&["landscaping"])
                                .inc_by(events.len() as u64);
                            SERVER_SENT_EVENT_STREAMS_TOTAL
                                .with_label_values(&["landscaping"])
                                .inc();
                        }
                    }
                }
            }
            GameResponse::ActionError { .. } => {}
            GameResponse::Login { .. } => {}
        }
    }

    pub fn terminate(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[derive(Default)]
struct Trip {
    trips_id: usize,
    trips: HashMap<usize, Instant>,
}

impl Trip {
    pub fn start(&mut self) -> usize {
        self.trips_id += 1;
        self.trips.insert(self.trips_id, Instant::now());
        self.trips_id
    }

    pub fn finish(&mut self, trip_id: usize) -> f64 {
        match self.trips.remove(&trip_id) {
            None => 0.0,
            Some(timer) => timer.elapsed().as_secs_f64(),
        }
    }
}

fn spawn_listener(
    running: Arc<AtomicBool>,
    config: Configuration,
    authorization: Sender<Player>,
    disconnection: Sender<String>,
) {
    let listener = move || {
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
        let heartbeat_timeout = Duration::from_secs(2);

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

                let mut player_requests = SyncReceiver {
                    reader: stream.try_clone().unwrap(),
                };
                let mut player_responses = SyncSender {
                    writer: stream.try_clone().unwrap(),
                };

                // authorization blocks new player connections,
                // (should be super fast)
                let request: Option<(usize, PlayerRequest)> = player_requests.receive();
                let player = match request {
                    Some((
                        bytes,
                        PlayerRequest::Login {
                            version,
                            player,
                            password,
                        },
                    )) => {
                        SERVER_RECEIVED_BYTES.inc_by(bytes as u64);

                        if version != API_VERSION {
                            warn!(
                                "Unable to authorize '{}' {}, version mismatch {} != {}",
                                player, peer, version, API_VERSION
                            );
                            let result = LoginResult::VersionMismatch;
                            let response = GameResponse::Login { result };
                            player_responses.send(&response).unwrap();
                            continue;
                        }

                        if password != config.password {
                            warn!(
                                "Unable to authorize '{}' {}, invalid password",
                                player, peer
                            );
                            let result = LoginResult::InvalidPassword;
                            let response = GameResponse::Login { result };
                            player_responses.send(&response).unwrap();
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

                let trip = Arc::new(RwLock::new(Trip::default()));

                let player_id = player.name.clone();
                let player_disconnection = disconnection.clone();
                let player_requests_trip = trip.clone();
                let player_requests = move || {
                    info!("Start player '{}' requests thread", player_id);
                    while let Some((bytes, request)) = player_requests.receive() {
                        SERVER_RECEIVED_BYTES.inc_by(bytes as u64);
                        SERVER_RECEIVED_REQUESTS_TOTAL.inc();
                        if let PlayerRequest::Trip { id } = request {
                            if let Ok(trip) = player_requests_trip.write().as_mut() {
                                let rtt = trip.finish(id);
                                SERVER_RTT_SECONDS.with_label_values(&[&player_id]).set(rtt);
                            } else {
                                error!("Player '{player_id}' responses thread panicked");
                                break;
                            }
                            continue;
                        }
                        if requests_sender.send(request).is_err() {
                            error!("Unable to receive request, server not working");
                            break;
                        }
                    }
                    info!("Stop player '{}' requests thread", player_id);

                    if player_disconnection.send(player_id).is_err() {
                        error!("Unable to disconnect player, server not working")
                    }
                };
                thread::Builder::new()
                    .name("player_requests".into())
                    .spawn(player_requests)
                    .unwrap();

                let player_id = player.name.clone();
                let player_responses = move || {
                    info!("Start player '{player_id}' responses thread");

                    let response = GameResponse::Login {
                        result: LoginResult::Success,
                    };
                    player_responses.send(&response).unwrap();

                    loop {
                        let response = match responses_receiver.recv_timeout(heartbeat_timeout) {
                            Ok(response) => response,
                            Err(RecvTimeoutError::Timeout) => {
                                encode(&GameResponse::Heartbeat).unwrap()
                            }
                            Err(RecvTimeoutError::Disconnected) => {
                                error!("Unable to send response, connection lost");
                                break;
                            }
                        };
                        if let Ok(trip) = trip.write().as_mut() {
                            let id = trip.start();
                            let response = GameResponse::Trip { id };
                            if player_responses.send(&response).is_none() {
                                error!("Unable to send trip '{player_id}'");
                                break;
                            }
                        } else {
                            error!("Player '{player_id}' requests thread panicked");
                            break;
                        }
                        match player_responses.send_body(response) {
                            Some(bytes) => {
                                SERVER_SENT_BYTES
                                    .with_label_values(&[&player_id])
                                    .inc_by(bytes as u64);
                                SERVER_SENT_RESPONSES_TOTAL
                                    .with_label_values(&[&player_id])
                                    .inc();
                            }
                            None => {
                                error!("Unable to send response, network error");
                                break;
                            }
                        }
                    }
                    info!("Stop player '{}' responses thread", player_id);
                };
                thread::Builder::new()
                    .name("player_responses".into())
                    .spawn(player_responses)
                    .unwrap();

                if authorization.send(player).is_err() {
                    error!("Unable to authorize {}, server not working", peer);
                    break;
                }
            }
        }
        info!("Server listener terminated")
    };
    thread::Builder::new()
        .name("listener".into())
        .spawn(listener)
        .unwrap();
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
