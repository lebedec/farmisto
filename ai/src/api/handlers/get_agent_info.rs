


use anyhow::Result;



use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_agent_info(nature: &Nature, id: usize) -> Result<ProcedureResult> {
    let agent = nature.get_creature_agent(id).unwrap();
    let creature = nature
        .creature_agents
        .iter()
        .find(|agent| agent.entity.id == id)
        .unwrap();
    let result = ProcedureResult::GetAgentInfo {
        thinking: creature.thinking.clone(),
        targeting: creature.targeting.clone(),
        position: agent.position,
        interaction: agent.interaction,
        radius: agent.radius,
        health: agent.health,
        thirst: agent.thirst,
        weight: agent.weight,
        hunger: agent.hunger,
        age: agent.age,
        daytime: agent.daytime,
        timestamps: agent.timestamps.clone(),
        cooldowns: agent.cooldowns.clone(),
        disabling: agent.disabling.clone(),
    };
    Ok(result)
}
