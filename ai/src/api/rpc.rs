use game::raising::Behaviour;
use std::collections::HashMap;

use crate::fauna::Targeting;
use crate::{Behaviours, Thinking};

#[derive(serde::Deserialize)]
pub enum Procedure {
    GetAgentInfo { id: usize },
    GetAgentThinking { id: usize },
    GetAgents {},
    GetBehaviours {},
    SaveBehaviours { behaviours: Behaviours },
}

#[derive(serde::Serialize)]
pub enum ProcedureResult {
    GetAgentInfo {
        thinking: Thinking,
        targeting: Targeting,
        position: [f32; 2],
        radius: f32,
        health: f32,
        thirst: f32,
        hunger: f32,
        daytime: f32,
        timestamps: HashMap<Behaviour, f32>,
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
