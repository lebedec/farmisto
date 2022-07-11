use crate::engine::{MeshAsset, TextureAsset};
use crate::Assets;
use datamap::WithContext;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone)]
pub struct PropsAsset {
    data: Arc<RefCell<PropsAssetData>>,
}

pub struct PropsAssetData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}

impl PropsAsset {
    #[inline]
    pub fn texture(&self) -> TextureAsset {
        self.data.borrow().texture.clone()
    }

    #[inline]
    pub fn mesh(&self) -> MeshAsset {
        self.data.borrow().mesh.clone()
    }
}

// TODO: autogenerate
impl WithContext for PropsAssetData {
    type Context = Assets;

    fn parse(
        row: &rusqlite::Row,
        id: usize,
        context: &mut Self::Context,
        connection: &rusqlite::Connection,
    ) -> Result<Self, rusqlite::Error> {
        let texture: String = row.get("texture")?;
        let mesh: String = row.get("mesh")?;
        Ok(Self {
            texture: context.texture(texture),
            mesh: context.mesh(mesh),
        })
    }
}

// TODO: autogenerate
impl From<Arc<RefCell<PropsAssetData>>> for PropsAsset {
    fn from(data: Arc<RefCell<PropsAssetData>>) -> Self {
        Self { data }
    }
}
