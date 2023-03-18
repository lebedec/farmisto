use anyhow::Result;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_views(nature: &Nature) -> Result<ProcedureResult> {
    let crops: Vec<usize> = nature.crops.iter().map(|crop| crop.entity.id).collect();
    let result = ProcedureResult::GetViews { crops };
    Ok(result)
}
