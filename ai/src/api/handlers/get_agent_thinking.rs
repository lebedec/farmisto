use serde_json::json;

use crate::Nature;

pub fn get_agent_thinking(
    nature: &Nature,
    resource: Vec<usize>,
) -> Result<Vec<u8>, serde_json::Error> {
    let id = resource[0];
    let creature = nature
        .creature_agents
        .iter()
        .find(|agent| agent.creature.id == id)
        .unwrap();
    let payload = json!({
        "animal_crop": creature.animal_crop,
    });
    serde_json::to_vec(&payload)
}
