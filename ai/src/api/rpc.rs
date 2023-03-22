use crate::{Behaviours, Thinking};

#[derive(serde::Deserialize)]
pub enum Procedure {
    GetAgentInfo {id: usize },
    GetAgentThinking { id: usize },
    GetAgents {},
    GetBehaviours {},
    GetViews { id: usize },
    SaveBehaviours { behaviours: Behaviours },
}

#[derive(serde::Serialize)]
pub enum ProcedureResult {
    GetAgentInfo {
        thinking: Thinking,
        crops: Vec<usize>,
        tiles: Vec<[usize; 2]>,
        position: [f32; 2],
        radius: usize
    },
    GetAgentThinking {
        thinking: Thinking,
    },
    GetAgents {
        creatures: Vec<usize>,
    },
    GetBehaviours {
        behaviours: Behaviours,
    },
    GetViews {
        crops: Vec<usize>,
        tiles: Vec<[usize; 2]>,
    },
    SaveBehaviours {},
}
