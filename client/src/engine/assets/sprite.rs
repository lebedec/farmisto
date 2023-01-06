use crate::engine::assets::asset::Asset;
use crate::engine::{SamplerAsset, TextureAsset};

pub type SpriteAsset = Asset<SpriteAssetData>;

pub struct SpriteAssetData {
    pub texture: TextureAsset,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub sampler: SamplerAsset,
    pub pivot: [f32; 2],
}
