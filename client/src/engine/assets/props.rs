use crate::engine::{MeshAsset, PathRef, PrefabError, TextureAsset};
use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct PropsAsset {
    data: Arc<RefCell<PropsAssetData>>,
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

    #[inline]
    pub fn update(&mut self, data: PropsAssetData) {
        let mut this = self.data.borrow_mut();
        *this = data;
    }

    pub fn from_data(data: Arc<RefCell<PropsAssetData>>) -> Self {
        Self { data }
    }
}

pub struct PropsAssetData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}

#[derive(serde::Deserialize)]
pub struct PropsAssetConfig {
    pub texture: PathRef,
    pub mesh: PathRef,
}

impl PropsAssetConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PrefabError> {
        let file = File::open(path).map_err(PrefabError::Io)?;
        serde_yaml::from_reader(file).map_err(PrefabError::Yaml)
    }
}
