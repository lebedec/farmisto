use crate::engine::assets::asset::Asset;
use crate::engine::{SamplerAsset, SpriteAsset, SpriteAssetData, TextureAsset};

pub type TilesetAsset = Asset<TilesetAssetData>;

pub struct TilesetAssetData {
    pub texture: TextureAsset,
    pub sampler: SamplerAsset,
    pub tiles: Vec<SpriteAsset>,
}

#[derive(serde::Deserialize)]
pub struct TilesetItem {
    pub src: [f32; 2],
    pub size: [f32; 2],
    pub pivot: [f32; 2],
}
