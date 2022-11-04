use crate::engine::assets::generic::Asset;
use crate::engine::TextureAsset;

pub type SpriteAsset = Asset<SpriteAssetData>;

pub struct SpriteAssetData {
    pub texture: TextureAsset,
}