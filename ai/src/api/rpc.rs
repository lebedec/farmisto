use crate::{Behaviours, Thinking};

#[derive(serde::Deserialize)]
pub enum Procedure {
    GetAgentThinking { id: usize },
    GetAgents {},
    GetBehaviours {},
    GetViews {},
    SaveBehaviours { behaviours: Behaviours },
}

#[derive(serde::Serialize)]
pub enum ProcedureResult {
    GetAgentThinking { animal_crop: Thinking },
    GetAgents { creatures: Vec<usize> },
    GetBehaviours { behaviours: Behaviours },
    GetViews { crops: Vec<usize> },
    SaveBehaviours {},
}
