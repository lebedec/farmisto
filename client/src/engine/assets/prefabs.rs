use crate::engine::{MeshAsset, TextureAsset};
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct PathRef {
    #[serde(rename = "$ref")]
    path: PathBuf,
}

impl PathRef {
    #[inline]
    pub fn resolve<P: AsRef<Path>>(&self, source: P) -> PathBuf {
        source.as_ref().to_path_buf().join(&self.path)
    }
}

#[derive(Clone)]
pub struct TreePrefab {
    data: Arc<RefCell<TreePrefabData>>,
}

impl TreePrefab {
    #[inline]
    pub fn texture(&self) -> TextureAsset {
        self.data.borrow().texture.clone()
    }

    #[inline]
    pub fn mesh(&self) -> MeshAsset {
        self.data.borrow().mesh.clone()
    }

    #[inline]
    pub fn update(&mut self, data: TreePrefabData) {
        let mut this = self.data.borrow_mut();
        *this = data;
    }

    pub fn from_data(data: Arc<RefCell<TreePrefabData>>) -> Self {
        Self { data }
    }
}

pub struct TreePrefabData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}

#[derive(serde::Deserialize)]
pub struct TreePrefabConfig {
    pub texture: PathRef,
    pub mesh: PathRef,
}

impl TreePrefabConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PrefabError> {
        let file = File::open(path).map_err(PrefabError::Io)?;
        serde_yaml::from_reader(file).map_err(PrefabError::Yaml)
    }
}

#[derive(Debug)]
pub enum PrefabError {
    Io(io::Error),
    Yaml(serde_yaml::Error),
}
