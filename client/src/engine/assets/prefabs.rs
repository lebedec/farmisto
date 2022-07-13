use crate::engine::assets::generic::Asset;
use crate::engine::{MeshAsset, PropsAsset, TextureAsset};
use glam::Vec3;

pub type FarmlandAsset = Asset<FarmlandAssetData>;

#[derive(datamap::AssetData)]
pub struct FarmlandAssetData {
    #[prefetch]
    pub props: Vec<FarmlandAssetPropItem>,
}

#[derive(datamap::AssetData)]
pub struct FarmlandAssetPropItem {
    pub id: usize,
    #[parent]
    pub farmland: usize,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    #[context]
    pub asset: PropsAsset,
}

impl FarmlandAssetPropItem {
    #[inline]
    pub fn position(&self) -> Vec3 {
        Vec3::from(self.position)
    }

    #[inline]
    pub fn rotation(&self) -> Vec3 {
        Vec3::from(self.rotation)
    }

    #[inline]
    pub fn scale(&self) -> Vec3 {
        Vec3::from(self.scale)
    }
}

pub type TreeAsset = Asset<TreeAssetData>;

#[derive(datamap::AssetData)]
pub struct TreeAssetData {
    #[context]
    pub texture: TextureAsset,
    #[context]
    pub mesh: MeshAsset,
}

pub type FarmerAsset = Asset<FarmerAssetData>;

#[derive(datamap::AssetData)]
pub struct FarmerAssetData {
    #[context]
    pub texture: TextureAsset,
    #[context]
    pub mesh: MeshAsset,
}
