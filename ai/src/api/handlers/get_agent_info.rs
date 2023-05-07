use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;

use game::math::VectorMath;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_agent_info(nature: &Nature, id: usize) -> Result<ProcedureResult> {
    let agent = nature.get_creature_agent(id).unwrap();
    let crops: Vec<usize> = nature.crops.iter().map(|crop| crop.entity.id).collect();
    let tiles = nature.get_tiles_around(agent.space, agent.position.to_tile(), agent.radius);
    let creature = nature
        .creature_agents
        .iter()
        .find(|agent| agent.creature.id == id)
        .unwrap();
    // let history = HashMap::from_iter(agent.history.iter().map(|(key, value)| {
    //     let [set, behaviour, decision] = key;
    //     let offset = value.elapsed().as_secs();
    //     let time = SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //     let d = time - offset;
    //     (format!("{set}:{behaviour}:{decision}"), d)
    // }));
    let result = ProcedureResult::GetAgentInfo {
        thinking: creature.thinking.clone(),
        crops,
        tiles,
        position: agent.position,
        radius: agent.radius,
        history: HashMap::default(),
    };
    Ok(result)
}
