use std::fs;

use crate::api::rpc::ProcedureResult;
use crate::Behaviours;
use anyhow::Result;

pub fn get_behaviours() -> Result<ProcedureResult> {
    let path = "./assets/ai/nature.json";
    let data = fs::read(path)?;
    let behaviours: Behaviours = serde_json::from_slice(&data)?;
    Ok(ProcedureResult::GetBehaviours { behaviours })
}
