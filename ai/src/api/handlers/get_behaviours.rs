use std::fs;

use serde_json::json;

use crate::{Behaviours, Nature};

pub fn get_behaviours(
    _nature: &Nature,
    _resource: Vec<usize>,
) -> Result<Vec<u8>, serde_json::Error> {
    let data = fs::read("./assets/ai/nature.json").unwrap();
    let behaviours: Behaviours = serde_json::from_slice(&data)?;
    let payload = json!({
        "version": 42,
        "behaviours": behaviours
    });
    serde_json::to_vec(&payload)
}
