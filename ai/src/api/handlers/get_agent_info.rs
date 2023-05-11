use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;

use game::math::VectorMath;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_agent_info(nature: &Nature, id: usize) -> Result<ProcedureResult> {
    let agent = nature.get_creature_agent(id).unwrap();
    let crops: Vec<usize> = nature.crops.iter().map(|crop| crop.entity.id).collect();
    let foods: Vec<usize> = nature.foods.values().map(|food| food.item.0).collect();
    let tiles = nature.get_tiles_around(
        agent.farmland.space,
        agent.position.to_tile(),
        agent.radius as usize,
    );
    let creature = nature
        .creature_agents
        .iter()
        .find(|agent| agent.entity.id == id)
        .unwrap();
    let result = ProcedureResult::GetAgentInfo {
        thinking: creature.thinking.clone(),
        crops,
        tiles,
        foods,
        me: vec![agent.entity.id],
        position: agent.position,
        radius: agent.radius,
        health: agent.health,
        thirst: agent.thirst,
        hunger: agent.hunger,
        timestamps: agent.timestamps.clone(),
    };
    Ok(result)
}
