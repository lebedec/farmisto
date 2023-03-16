use ash::vk;

use crate::assets::Asset;

pub type SamplerAsset = Asset<SamplerAssetData>;

pub struct SamplerAssetData {
    pub handle: vk::Sampler,
}
