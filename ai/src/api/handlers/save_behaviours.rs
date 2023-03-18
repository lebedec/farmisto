use std::fs;

use crate::api::rpc::ProcedureResult;
use crate::Behaviours;
use anyhow::Result;

pub fn save_behaviours(behaviours: Behaviours) -> Result<ProcedureResult> {
    let path = "./assets/ai/nature.json";
    let data = serde_json::to_vec_pretty(&behaviours)?;
    fs::write(path, data)?;
    Ok(ProcedureResult::SaveBehaviours {})
}
