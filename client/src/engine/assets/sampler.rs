use crate::engine::assets::asset::Asset;
use ash::vk;

pub type SamplerAsset = Asset<SamplerAssetData>;

pub struct SamplerAssetData {
    pub handle: vk::Sampler,
}
