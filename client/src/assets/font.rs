use crate::assets::Asset;
use fontdue::Font;

pub type FontAsset = Asset<FontAssetData>;

pub struct FontAssetData {
    pub font_type: Font,
}
