use std::fs;

use crate::api::rpc::ProcedureResult;
use crate::api::rpc::ProcedureResult::LoadBehaviours;
use crate::Behaviours;

pub fn get_behaviours() -> Result<ProcedureResult, serde_json::Error> {
    let data = fs::read("./assets/ai/nature.json").unwrap();
    let behaviours: Behaviours = serde_json::from_slice(&data)?;
    Ok(LoadBehaviours { behaviours })
}
