use crate::engine::{TextureAsset, TextureAssetData};
use ash::{vk, Device};
use image::{load_from_memory, DynamicImage};
use log::debug;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct AssetServer {
    textures_default: Arc<RefCell<TextureAssetData>>,
    textures: HashMap<PathBuf, TextureAsset>,
    loading_requests: Arc<RwLock<Vec<AssetRequest>>>,
}

pub struct AssetRequest {
    pub path: PathBuf,
    pub kind: AssetKind,
}

#[derive(Debug)]
pub enum AssetKind {
    Texture,
    Shader,
}

impl AssetServer {
    pub fn new(
        device: Device,
        pool: vk::CommandPool,
        queue: vk::Queue,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        let textures_default = Arc::new(RefCell::new(TextureAssetData::create_and_read_image(
            &device,
            pool,
            queue,
            device_memory_properties,
            load_from_memory(include_bytes!("./fallback/texture.png")).unwrap(),
        )));

        Self {
            textures_default,
            textures: Default::default(),
            loading_requests: Arc::new(Default::default()),
        }
    }

    pub fn texture<P: AsRef<Path>>(&mut self, path: P) -> TextureAsset {
        let path = path.as_ref().to_path_buf();
        if let Some(texture) = self.textures.get(&path) {
            return texture.clone();
        }
        let texture = TextureAsset::from_data(self.textures_default.clone());
        self.textures.insert(path.clone(), texture.clone());
        self.require_update(AssetKind::Texture, path);
        texture
    }

    fn require_update(&mut self, kind: AssetKind, path: PathBuf) {
        debug!("Require update {:?} {:?}", kind, path.to_str());
        let mut requests = self.loading_requests.write().unwrap();
        requests.push(AssetRequest { path, kind });
    }
}
