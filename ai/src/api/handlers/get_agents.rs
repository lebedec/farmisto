use serde_json::json;

use crate::Nature;

pub fn get_agents(nature: &Nature, _resource: Vec<usize>) -> Result<Vec<u8>, serde_json::Error> {
    let creatures: Vec<usize> = nature
        .creature_agents
        .iter()
        .map(|agent| agent.creature.id)
        .collect();
    let payload = json!({
        "creatures": creatures,
    });
    serde_json::to_vec(&payload)
}
