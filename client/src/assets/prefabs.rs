use crate::assets::Asset;
use crate::assets::{
    PropsAsset, SamplerAsset, SpineAsset, SpriteAsset, TextureAsset, TilesetAsset,
};

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
}

pub type FarmerAsset = Asset<FarmerAssetData>;

pub struct FarmerAssetData {
    pub texture: TextureAsset,
}

pub type ItemAsset = Asset<ItemAssetData>;

pub struct ItemAssetData {
    pub sprite: SpriteAsset,
    pub quantitative: Option<TilesetAsset>,
}

pub type CropAsset = Asset<CropAssetData>;

pub struct CropAssetData {
    pub sprout: SpineAsset,
    pub vegetating: SpineAsset,
    pub flowering: SpineAsset,
    pub ripening: SpineAsset,
    pub withering: SpineAsset,
    pub damage_mask: TextureAsset,
}

pub type CreatureAsset = Asset<CreatureAssetData>;

pub struct CreatureAssetData {
    pub spine: SpineAsset,
    pub coloration: TextureAsset,
}

pub type BuildingMaterialAsset = Asset<BuildingMaterialAssetData>;

pub struct BuildingMaterialAssetData {
    pub roof: TextureAsset,
    pub roof_sampler: SamplerAsset,
    pub floor: TextureAsset,
    pub floor_sampler: SamplerAsset,
    pub walls: TilesetAsset,
    pub walls_transparency: TilesetAsset,
}

pub type DoorAsset = Asset<DoorAssetData>;

pub struct DoorAssetData {
    pub sprites: TilesetAsset,
}

pub type RestAsset = Asset<RestAssetData>;

pub struct RestAssetData {
    pub sprites: TilesetAsset,
}

pub type CementerAsset = Asset<CementerAssetData>;

pub struct CementerAssetData {
    pub sprites: TilesetAsset,
}

pub type ComposterAsset = Asset<ComposterAssetData>;

pub struct ComposterAssetData {
    pub sprites: TilesetAsset,
}

pub type CorpseAsset = Asset<CorpseAssetData>;

pub struct CorpseAssetData {
    pub sprite: SpriteAsset,
}
