use crate::engine::base::Queue;
use crate::engine::{MeshAsset, MeshAssetData, TextureAsset, TextureAssetData};
use ash::{vk, Device};
use image::load_from_memory;
use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct AssetManager {
    loading_requests: Arc<RwLock<Vec<AssetRequest>>>,
    loading_result: Receiver<AssetPayload>,

    textures_default: TextureAssetData,
    textures: HashMap<PathBuf, TextureAsset>,

    meshes_default: MeshAssetData,
    meshes: HashMap<PathBuf, MeshAsset>,

    pub texture_set_layout: vk::DescriptorSetLayout,
}

pub struct AssetRequest {
    pub path: PathBuf,
    pub kind: AssetKind,
}

pub enum AssetPayload {
    Texture {
        path: PathBuf,
        data: TextureAssetData,
    },
    Mesh {
        path: PathBuf,
        data: MeshAssetData,
    },
}

#[derive(Debug)]
pub enum AssetKind {
    Texture(vk::DescriptorSetLayout),
    Shader,
    Mesh,
}

impl AssetManager {
    pub fn new(
        device: Device,
        pool: vk::CommandPool,
        queue: Arc<Queue>,
        tex_descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Self {
        let textures_default = TextureAssetData::create_and_read_image(
            &device,
            pool,
            queue.clone(),
            load_from_memory(include_bytes!("./fallback/texture.png")).unwrap(),
            tex_descriptor_set_layout,
        );
        let textures = HashMap::new();

        let meshes_default = MeshAssetData::fallback(&queue).unwrap();
        let meshes = HashMap::new();

        let loading_requests = Arc::new(RwLock::new(Vec::<AssetRequest>::new()));
        let (loading, loading_result) = channel();

        let loaders = 4;
        for loader in 0..loaders {
            let loaders_requests = loading_requests.clone();
            let loader_queue = queue.clone();
            let loader_result = loading.clone();

            let loader_device = device.clone();
            thread::spawn(move || {
                info!("[loader-{}] Start", loader);
                loop {
                    let request = { loaders_requests.write().unwrap().pop() };
                    if let Some(request) = request {
                        info!(
                            "[loader-{}] Load {:?} {:?}",
                            loader,
                            request.kind,
                            request.path.to_str()
                        );
                        let path = request.path.clone();
                        match request.kind {
                            AssetKind::Texture(descriptor_set_layout) => {
                                loader_result
                                    .send(AssetPayload::Texture {
                                        path: request.path.clone(),
                                        data: TextureAssetData::create_and_read_image(
                                            &loader_device,
                                            pool,
                                            loader_queue.clone(),
                                            image::open(request.path).unwrap(),
                                            descriptor_set_layout,
                                        ),
                                    })
                                    .unwrap();
                            }
                            AssetKind::Shader => {}
                            AssetKind::Mesh => {
                                match MeshAssetData::from_json_file(&loader_queue, &path) {
                                    Ok(data) => {
                                        loader_result.send(AssetPayload::Mesh { path, data });
                                    }
                                    Err(error) => {
                                        info!(
                                            "[loader-{}] Unable to load {:?} {:?}, {:?}",
                                            loader,
                                            request.kind,
                                            path.to_str(),
                                            error
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        thread::sleep(Duration::from_millis(150))
                    }
                }
            });
        }

        Self {
            textures_default,
            textures,
            meshes_default,
            loading_requests,
            loading_result,
            texture_set_layout: tex_descriptor_set_layout,
            meshes,
        }
    }

    pub fn texture<P: AsRef<Path>>(
        &mut self,
        path: P,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> TextureAsset {
        let path = path.as_ref().to_path_buf();
        if let Some(texture) = self.textures.get(&path) {
            return texture.clone();
        }
        let texture =
            TextureAsset::from_data(Arc::new(RefCell::new(self.textures_default.clone())));
        self.textures.insert(path.clone(), texture.clone());
        self.require_update(AssetKind::Texture(descriptor_set_layout), path);
        texture
    }

    pub fn mesh<P: AsRef<Path>>(&mut self, path: P) -> MeshAsset {
        let path = path.as_ref().to_path_buf();
        if let Some(mesh) = self.meshes.get(&path) {
            return mesh.clone();
        }
        let mesh = MeshAsset::from_data(Arc::new(RefCell::new(self.meshes_default.clone())));
        self.meshes.insert(path.clone(), mesh.clone());
        self.require_update(AssetKind::Mesh, path);
        mesh
    }

    fn require_update(&mut self, kind: AssetKind, path: PathBuf) {
        info!("Require update {:?} {:?}", kind, path.to_str());
        let mut requests = self.loading_requests.write().unwrap();
        requests.push(AssetRequest { path, kind });
    }

    pub fn update(&mut self) {
        for payload in self.loading_result.try_iter() {
            match payload {
                AssetPayload::Texture { path, data } => match self.textures.get_mut(&path) {
                    None => {
                        error!(
                            "Unable to update texture {:?}, not registered",
                            path.to_str()
                        );
                    }
                    Some(texture) => {
                        info!("Update texture {:?}", path.to_str());
                        texture.update(data);
                    }
                },
                AssetPayload::Mesh { path, data } => match self.meshes.get_mut(&path) {
                    None => {
                        error!("Unable to update mesh {:?}, not registered", path.to_str());
                    }
                    Some(mesh) => {
                        info!("Update mesh {:?}", path.to_str());
                        mesh.update(data);
                    }
                },
            }
        }
    }
}
