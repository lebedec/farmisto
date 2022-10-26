use crate::engine::assets::generic::Asset;
use crate::engine::{MeshAsset, TextureAsset};

pub type PropsAsset = Asset<PropsAssetData>;

pub struct PropsAssetData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}
