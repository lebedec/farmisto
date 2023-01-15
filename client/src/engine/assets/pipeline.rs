use crate::engine::assets::asset::Asset;
use crate::engine::ShaderAsset;

pub type PipelineAsset = Asset<PipelineAssetData>;

pub struct PipelineAssetData {
    pub name: String,
    pub fragment: ShaderAsset,
    pub vertex: ShaderAsset,
    pub changed: bool,
}
