use crate::{Behaviours, Thinking};

#[derive(serde::Deserialize)]
pub enum Procedure {
    LoadBehaviours {},
    GetAgentThinking { id: usize },
}

#[derive(serde::Serialize)]
pub enum ProcedureResult {
    Nothing,
    LoadBehaviours { behaviours: Behaviours },
    GetAgentThinking { animal_crop: Thinking },
}
