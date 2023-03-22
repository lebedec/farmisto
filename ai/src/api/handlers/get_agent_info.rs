use anyhow::Result;

use game::math::VectorMath;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_agent_info(nature: &Nature, id: usize) -> Result<ProcedureResult> {
    let agent = nature.get_creature_agent(id).unwrap();
    let crops: Vec<usize> = nature.crops.iter().map(|crop| crop.entity.id).collect();
    let game_tiles = nature.tiles.get(&agent.space).unwrap();
    let tiles = Nature::get_empty_tiles(game_tiles, agent.position.to_tile(), agent.radius);
    let creature = nature
        .creature_agents
        .iter()
        .find(|agent| agent.creature.id == id)
        .unwrap();
    let result = ProcedureResult::GetAgentInfo {
        thinking: creature.thinking.clone(),
        crops,
        tiles,
        position: agent.position,
        radius: agent.radius,
    };
    Ok(result)
}
