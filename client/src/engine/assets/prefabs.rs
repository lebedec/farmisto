use crate::engine::assets::asset::Asset;
use crate::engine::{MeshAsset, PropsAsset, SamplerAsset, TextureAsset};

pub type FarmlandAsset = Asset<FarmlandAssetData>;

pub struct FarmlandAssetData {
    pub props: Vec<FarmlandAssetPropItem>,
    pub texture: TextureAsset,
    pub sampler: SamplerAsset,
}

pub struct FarmlandAssetPropItem {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub asset: PropsAsset,
}

pub type TreeAsset = Asset<TreeAssetData>;

pub struct TreeAssetData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}

pub type FarmerAsset = Asset<FarmerAssetData>;

pub struct FarmerAssetData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}
