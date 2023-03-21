use anyhow::Result;
use game::math::VectorMath;

use crate::api::rpc::ProcedureResult;
use crate::Nature;

pub fn get_views(id: usize, nature: &Nature) -> Result<ProcedureResult> {
    let agent = nature.get_creature_agent(id).unwrap();
    let crops: Vec<usize> = nature.crops.iter().map(|crop| crop.entity.id).collect();
    let game_tiles = nature.tiles.get(&agent.space).unwrap();
    let tiles = Nature::get_empty_tiles(game_tiles, agent.position.to_tile(), agent.radius);
    let result = ProcedureResult::GetViews { crops, tiles };
    Ok(result)
}
