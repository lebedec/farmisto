use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use log::{error, info};
use tungstenite::Message;

use crate::api::handlers;
use crate::api::rpc::Procedure::{
    GetAgentInfo, GetAgentThinking, GetAgents, GetBehaviours, SaveBehaviours,
};
use crate::api::rpc::{Procedure, ProcedureResult};
use crate::Nature;

pub fn handle_rpc(
    nature: &Nature,
    calls: Receiver<Procedure>,
    sender: &Sender<ProcedureResult>,
) -> anyhow::Result<Receiver<Procedure>> {
    for procedure in calls.try_iter() {
        let result = match procedure {
            GetAgentInfo { id } => handlers::get_agent_info(nature, id),
            GetAgentThinking { id } => handlers::get_agent_thinking(nature, id),
            GetAgents { .. } => handlers::get_agents(nature),
            GetBehaviours { .. } => handlers::get_behaviours(),
            SaveBehaviours { behaviours } => handlers::save_behaviours(behaviours),
        };
        sender.send(result?)?;
    }
    Ok(calls)
}

pub fn serve_web_socket(call: Sender<Procedure>, results: Receiver<ProcedureResult>) {
    let address = "0.0.0.0:9098";
    info!("Starts listen web sockets on on {address}");
    let server = TcpListener::bind(address).unwrap();
    for stream in server.incoming() {
        info!("Accept new web socket");
        let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();

        while let Ok(msg) = websocket.read_message() {
            if let Message::Text(data) = msg {
                let procedure: Procedure = match serde_json::from_str(&data) {
                    Ok(procedure) => procedure,
                    Err(error) => {
                        error!("Unable to receive procedure call, {error}");
                        continue;
                    }
                };

                if call.send(procedure).is_err() {
                    error!("Unable to handle procedure call, AI thread terminated");
                    break;
                }

                for result in results.try_iter() {
                    let message = match serde_json::to_string(&result) {
                        Ok(message) => message,
                        Err(error) => {
                            error!("Unable to write result message, {error}");
                            continue;
                        }
                    };
                    if let Err(error) = websocket.write_message(Message::text(message)) {
                        error!("Unable to send message, {error}");
                        break;
                    }
                }
            }
        }
        info!("Close web socket");
    }
}
