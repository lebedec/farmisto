use std::sync::Arc;

use rusty_spine::{AnimationStateData, SkeletonData};

use crate::assets::Asset;
use crate::assets::TextureAsset;

pub type SpineAsset = Asset<SpineAssetData>;

pub struct SpineAssetData {
    pub skeleton: Arc<SkeletonData>,
    pub animation: Arc<AnimationStateData>,
    pub atlas: TextureAsset,
}
