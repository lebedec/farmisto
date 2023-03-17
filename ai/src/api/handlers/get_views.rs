use serde_json::json;

use crate::Nature;

pub fn get_views(nature: &Nature, _resource: Vec<usize>) -> Result<Vec<u8>, serde_json::Error> {
    let crops: Vec<usize> = nature.crops.iter().map(|crop| crop.entity.id).collect();
    let payload = json!({
        "crops": crops,
    });
    serde_json::to_vec(&payload)
}
