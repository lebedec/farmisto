use crate::{Behaviours, Thinking};

#[derive(serde::Deserialize)]
pub enum Procedure {
    GetAgentThinking { id: usize },
    GetAgents {},
    GetBehaviours {},
    GetViews { id: usize },
    SaveBehaviours { behaviours: Behaviours },
}

#[derive(serde::Serialize)]
pub enum ProcedureResult {
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
