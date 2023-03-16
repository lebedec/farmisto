use crate::assets::Asset;
use crate::assets::TextureAsset;

pub type PropsAsset = Asset<PropsAssetData>;

pub struct PropsAssetData {
    pub texture: TextureAsset,
}
