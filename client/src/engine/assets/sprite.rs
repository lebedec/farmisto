use crate::engine::assets::asset::Asset;
use crate::engine::TextureAsset;

pub type SpriteAsset = Asset<SpriteAssetData>;

pub struct SpriteAssetData {
    pub texture: TextureAsset,
}
