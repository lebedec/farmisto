use std::net::TcpListener;
use std::sync::{Arc, RwLock};

use log::{error, info};
use tungstenite::Message;

use crate::api::handlers;
use crate::api::rpc::Procedure;
use crate::api::rpc::Procedure::{
    GetAgentThinking, GetAgents, GetBehaviours, GetViews, SaveBehaviours,
};
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
                let nature = &nature.read().unwrap();
                let result = match procedure {
                    GetAgentThinking { id } => handlers::get_agent_thinking(nature, id),
                    GetAgents { .. } => handlers::get_agents(nature),
                    GetBehaviours { .. } => handlers::get_behaviours(),
                    GetViews { id } => handlers::get_views(id, nature),
                    SaveBehaviours { behaviours } => handlers::save_behaviours(behaviours),
                };
                let message = match result {
                    Ok(result) => serde_json::to_string(&result).unwrap(),
                    Err(error) => {
                        error!("Unable to handle message because of error, {error}");
                        continue;
                    }
                };
                if let Err(error) = websocket.write_message(Message::text(message)) {
                    error!("Unable to send message, {error}");
                    break;
                }
            }
        }
        info!("Close web socket");
    }
}
