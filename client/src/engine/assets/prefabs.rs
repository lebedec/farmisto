use crate::engine::{MeshAsset, PropsAsset, TextureAsset};
use glam::Vec3;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
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
pub struct FarmlandPrefab {
    data: Arc<RefCell<FarmlandPrefabData>>,
}

impl FarmlandPrefab {
    #[inline]
    pub fn props(&self) -> Vec<Transform<PropsAsset>> {
        // todo: remove clone
        self.data.borrow().props.clone()
    }

    #[inline]
    pub fn update(&mut self, data: FarmlandPrefabData) {
        let mut this = self.data.borrow_mut();
        *this = data;
    }

    pub fn from_data(data: Arc<RefCell<FarmlandPrefabData>>) -> Self {
        Self { data }
    }
}

#[derive(Clone)]
pub struct Transform<T> {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub entity: T,
}

pub struct FarmlandPrefabData {
    pub props: Vec<Transform<PropsAsset>>,
    pub config: FarmlandPrefabConfig,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PropsConfig {
    pub position: Option<[f32; 3]>,
    pub rotation: Option<[f32; 3]>,
    pub scale: Option<[f32; 3]>,
    pub asset: PathRef,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FarmlandPrefabConfig {
    pub props: Vec<PropsConfig>,
}

impl FarmlandPrefabConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PrefabError> {
        let file = File::open(path).map_err(PrefabError::Io)?;
        serde_yaml::from_reader(file).map_err(PrefabError::Yaml)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), PrefabError> {
        let file = File::open(path).map_err(PrefabError::Io)?;
        serde_yaml::to_writer(file, self).map_err(PrefabError::Yaml)
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
