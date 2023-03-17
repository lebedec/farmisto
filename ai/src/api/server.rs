use std::net::TcpListener;
use std::sync::{Arc, RwLock};

use log::{error, info};
use tungstenite::Message;

use crate::api::handlers;
use crate::api::rpc::Procedure::{GetAgentThinking, LoadBehaviours};
use crate::api::rpc::{Procedure, ProcedureResult};
use crate::Nature;

pub fn serve_web_socket(nature: Arc<RwLock<Nature>>) {
    let address = "0.0.0.0:9098";
    info!("Starts listen web sockets on on {address}");
    let server = TcpListener::bind(address).unwrap();
    for stream in server.incoming() {
        info!("Accept new web socket");
        let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
        while let Ok(msg) = websocket.read_message() {
            if let Message::Text(data) = msg {
                let procedure: Procedure = serde_json::from_str(&data).unwrap();
                let nature = nature.read().unwrap();
                let result = match procedure {
                    LoadBehaviours { .. } => handlers::get_behaviours().unwrap(),
                    GetAgentThinking { .. } => ProcedureResult::Nothing,
                };
                let text = serde_json::to_string(&result).unwrap();
                if let Err(error) = websocket.write_message(Message::text(text)) {
                    error!("Unable to send message, {error}");
                    break;
                }
            }
        }
        info!("Close web socket");
    }
}
