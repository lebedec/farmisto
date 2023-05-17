use game::raising::Behaviour;
use std::collections::{HashMap, HashSet};

use crate::fauna::Targeting;
use crate::{Behaviours, Thinking};

#[derive(serde::Deserialize)]
pub enum Procedure {
    GetAgentInfo { id: usize },
    GetAgentThinking { id: usize },
    GetAgents {},
    SaveBehaviours { behaviours: Behaviours },
}

#[derive(serde::Serialize)]
pub enum ProcedureResult {
    GetAgentInfo {
        thinking: Thinking,
        targeting: Targeting,
        position: [f32; 2],
        interaction: f32,
        radius: f32,
        health: f32,
        thirst: f32,
        weight: f32,
        hunger: f32,
        age: f32,
        daytime: f32,
        timestamps: HashMap<Behaviour, f32>,
        cooldowns: HashMap<String, f32>,
        disabling: HashSet<String>,
    },
    GetAgentThinking {
        thinking: Thinking,
    },
    GetAgents {
        creatures: Vec<usize>,
    },
    GetViews {
        crops: Vec<usize>,
        tiles: Vec<[usize; 2]>,
    },
    SaveBehaviours {},
}
