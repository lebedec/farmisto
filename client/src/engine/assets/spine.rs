use std::sync::Arc;

use rusty_spine::controller::SkeletonController;
use rusty_spine::{AnimationStateData, SkeletonData};

use crate::engine::assets::asset::Asset;

pub type SpineAsset = Asset<SpineAssetData>;

pub struct SpineAssetData {
    pub skeleton: Arc<SkeletonData>,
    pub animation: Arc<AnimationStateData>,
}
