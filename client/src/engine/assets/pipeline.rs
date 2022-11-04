use crate::engine::assets::generic::Asset;
use crate::engine::ShaderAsset;

pub type PipelineAsset = Asset<PipelineAssetData>;

pub struct PipelineAssetData {
    pub fragment: ShaderAsset,
    pub vertex: ShaderAsset,
}
