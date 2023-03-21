use anyhow::Result;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_agent_thinking(nature: &Nature, id: usize) -> Result<ProcedureResult> {
    let creature = nature
        .creature_agents
        .iter()
        .find(|agent| agent.creature.id == id)
        .unwrap();
    let result = ProcedureResult::GetAgentThinking {
        thinking: creature.thinking.clone(),
    };
    Ok(result)
}
