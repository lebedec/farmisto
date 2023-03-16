use crate::assets::Asset;
use crate::assets::ShaderAsset;

pub type PipelineAsset = Asset<PipelineAssetData>;

pub struct PipelineAssetData {
    pub name: String,
    pub fragment: ShaderAsset,
    pub vertex: ShaderAsset,
    pub changed: bool,
}
