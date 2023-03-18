use anyhow::Result;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_agents(nature: &Nature) -> Result<ProcedureResult> {
    let creatures: Vec<usize> = nature
        .creature_agents
        .iter()
        .map(|agent| agent.creature.id)
        .collect();
    let result = ProcedureResult::GetAgents { 
        creatures
    };
    Ok(result)
}
