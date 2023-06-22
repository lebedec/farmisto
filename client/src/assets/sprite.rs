use crate::assets::Asset;
use crate::assets::{SamplerAsset, TextureAsset};

pub type SpriteAsset = Asset<SpriteAssetData>;

pub struct SpriteAssetData {
    pub texture: TextureAsset,
    pub src: [f32; 2],
    pub size: [f32; 2],
    pub sampler: SamplerAsset,
    pub pivot: [f32; 2],
}
