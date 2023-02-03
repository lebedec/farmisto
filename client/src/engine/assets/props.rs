use crate::engine::assets::asset::Asset;
use crate::engine::TextureAsset;

pub type PropsAsset = Asset<PropsAssetData>;

pub struct PropsAssetData {
    pub texture: TextureAsset,
}
