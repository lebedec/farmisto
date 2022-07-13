use crate::engine::assets::generic::Asset;
use crate::engine::{MeshAsset, TextureAsset};

pub type PropsAsset = Asset<PropsAssetData>;

#[derive(datamap::AssetData)]
pub struct PropsAssetData {
    #[context]
    pub texture: TextureAsset,
    #[context]
    pub mesh: MeshAsset,
}
