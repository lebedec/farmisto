use crate::engine::assets::fs::{FileEvent, FileSystem};
use crate::engine::assets::prefabs::{TreePrefab, TreePrefabConfig, TreePrefabData};
use crate::engine::base::Queue;
use crate::engine::{
    FarmlandPrefab, FarmlandPrefabConfig, FarmlandPrefabData, MeshAsset, MeshAssetData, PropsAsset,
    PropsAssetConfig, PropsAssetData, ShaderAsset, ShaderAssetData, TextureAsset, TextureAssetData,
    Transform,
};
use crate::ShaderCompiler;
use ash::{vk, Device};
use glam::Vec3;
use log::{debug, error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{fs, thread};

pub struct Assets {
    loading_requests: Arc<RwLock<Vec<AssetRequest>>>,
    loading_result: Receiver<AssetPayload>,
    file_events: Arc<RwLock<HashMap<PathBuf, FileEvent>>>,

    textures_default: TextureAssetData,
    textures: HashMap<PathBuf, TextureAsset>,

    meshes_default: MeshAssetData,
    meshes: HashMap<PathBuf, MeshAsset>,

    shaders: HashMap<PathBuf, ShaderAsset>,

    farmlands: HashMap<PathBuf, FarmlandPrefab>,
    trees: HashMap<PathBuf, TreePrefab>,
    props: HashMap<PathBuf, PropsAsset>,

    queue: Arc<Queue>,
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
    Shader {
        path: PathBuf,
        data: ShaderAssetData,
    },
    Mesh {
        path: PathBuf,
        data: MeshAssetData,
    },
}

#[derive(Debug)]
pub enum AssetKind {
    Texture,
    Shader,
    Mesh,
    ShaderSrc,
}

impl Assets {
    pub fn new(device: Device, pool: vk::CommandPool, queue: Arc<Queue>) -> Self {
        info!(
            "Shader compiler version: {}",
            ShaderCompiler::new().version()
        );

        let textures_default = TextureAssetData::create_and_read_image(
            &device,
            pool,
            queue.clone(),
            include_bytes!("./fallback/texture.png"),
        );
        let textures = HashMap::new();

        let meshes_default = MeshAssetData::fallback(&queue).unwrap();
        let meshes = HashMap::new();

        let shaders = HashMap::new();

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
                            AssetKind::Texture => {
                                loader_result
                                    .send(AssetPayload::Texture {
                                        path: request.path.clone(),
                                        data: TextureAssetData::create_and_read_image(
                                            &loader_device,
                                            pool,
                                            loader_queue.clone(),
                                            &fs::read(request.path).unwrap(),
                                        ),
                                    })
                                    .unwrap();
                            }
                            AssetKind::Shader => {
                                let data = ShaderAssetData::from_spirv_file(&loader_queue, &path);
                                match data {
                                    Ok(data) => {
                                        loader_result
                                            .send(AssetPayload::Shader { path, data })
                                            .unwrap();
                                    }
                                    Err(error) => {
                                        error!(
                                            "[loader-{}] Unable to load {:?} {:?}, {:?}",
                                            loader,
                                            request.kind,
                                            path.to_str(),
                                            error
                                        );
                                    }
                                }
                            }
                            AssetKind::Mesh => {
                                let data = if path.extension().unwrap() == "space3" {
                                    MeshAssetData::from_space3(&loader_queue, &path)
                                } else {
                                    MeshAssetData::from_json_file(&loader_queue, &path)
                                };
                                match data {
                                    Ok(data) => {
                                        loader_result
                                            .send(AssetPayload::Mesh { path, data })
                                            .unwrap();
                                    }
                                    Err(error) => {
                                        error!(
                                            "[loader-{}] Unable to load {:?} {:?}, {:?}",
                                            loader,
                                            request.kind,
                                            path.to_str(),
                                            error
                                        );
                                    }
                                }
                            }
                            AssetKind::ShaderSrc => {
                                info!("[loader-{}] Compile shader {:?}", loader, path.to_str());
                                let compiler = ShaderCompiler::new();
                                compiler.compile_file(path);
                            }
                        }
                    } else {
                        thread::sleep(Duration::from_millis(15))
                    }
                }
            });
        }

        let file_events = FileSystem::watch();

        Self {
            textures_default,
            textures,
            meshes_default,
            loading_requests,
            loading_result,
            meshes,
            shaders,
            file_events,
            queue,
            farmlands: Default::default(),
            trees: Default::default(),
            props: Default::default(),
        }
    }

    pub fn shader<P: AsRef<Path>>(&mut self, path: P) -> ShaderAsset {
        let path = fs::canonicalize(path).unwrap();
        if let Some(shader) = self.shaders.get(&path) {
            return shader.clone();
        }
        let data = ShaderAssetData::from_spirv_file(&self.queue, &path).unwrap();
        let shader = ShaderAsset::from_data(Arc::new(RefCell::new(data)));
        self.shaders.insert(path.clone(), shader.clone());
        shader
    }

    pub fn texture<P: AsRef<Path>>(&mut self, path: P) -> TextureAsset {
        let path = fs::canonicalize(path).unwrap();
        if let Some(texture) = self.textures.get(&path) {
            return texture.clone();
        }
        let texture =
            TextureAsset::from_data(Arc::new(RefCell::new(self.textures_default.clone())));
        self.textures.insert(path.clone(), texture.clone());
        self.require_update(AssetKind::Texture, path);
        texture
    }

    pub fn mesh<P: AsRef<Path>>(&mut self, path: P) -> MeshAsset {
        let path = fs::canonicalize(path).unwrap();
        if let Some(mesh) = self.meshes.get(&path) {
            return mesh.clone();
        }
        let mesh = MeshAsset::from_data(Arc::new(RefCell::new(self.meshes_default.clone())));
        self.meshes.insert(path.clone(), mesh.clone());
        self.require_update(AssetKind::Mesh, path);
        mesh
    }

    pub fn tree<P: AsRef<Path>>(&mut self, path: P) -> TreePrefab {
        let path = fs::canonicalize(path).unwrap();
        if let Some(prefab) = self.trees.get(&path) {
            return prefab.clone();
        }
        let source = path.parent().unwrap();
        let config = TreePrefabConfig::from_file(&path).unwrap();
        let data = TreePrefabData {
            texture: self.texture(config.texture.resolve(&source)),
            mesh: self.mesh(config.mesh.resolve(&source)),
        };
        let prefab = TreePrefab::from_data(Arc::new(RefCell::new(data)));
        self.trees.insert(path, prefab.clone());
        prefab
    }

    pub fn farmland<P: AsRef<Path>>(&mut self, path: P) -> FarmlandPrefab {
        let path = fs::canonicalize(path).unwrap();
        if let Some(prefab) = self.farmlands.get(&path) {
            return prefab.clone();
        }
        let source = path.parent().unwrap();
        let config = FarmlandPrefabConfig::from_file(&path).unwrap();
        let data = FarmlandPrefabData {
            props: config
                .props
                .iter()
                .map(|config| Transform {
                    position: config
                        .position
                        .map(|values| Vec3::from(values))
                        .unwrap_or(Vec3::ZERO),
                    rotation: Default::default(),
                    scale: config
                        .scale
                        .map(|values| Vec3::from(values))
                        .unwrap_or(Vec3::ONE),
                    entity: self.props(config.asset.resolve(&source)),
                })
                .collect(),
        };
        let asset = FarmlandPrefab::from_data(Arc::new(RefCell::new(data)));
        self.farmlands.insert(path, asset.clone());
        asset
    }

    pub fn props<P: AsRef<Path>>(&mut self, path: P) -> PropsAsset {
        let path = fs::canonicalize(path).unwrap();
        if let Some(asset) = self.props.get(&path) {
            return asset.clone();
        }
        let source = path.parent().unwrap();
        let config = PropsAssetConfig::from_file(&path).unwrap();
        let data = PropsAssetData {
            texture: self.texture(config.texture.resolve(&source)),
            mesh: self.mesh(config.mesh.resolve(&source)),
        };
        let asset = PropsAsset::from_data(Arc::new(RefCell::new(data)));
        self.props.insert(path, asset.clone());
        asset
    }

    fn require_update(&mut self, kind: AssetKind, path: PathBuf) {
        info!("Require update {:?} {:?}", kind, path.to_str());
        let mut requests = self.loading_requests.write().unwrap();
        requests.push(AssetRequest { path, kind });
    }

    pub fn update(&mut self) {
        for (path, event) in self.observe_file_events() {
            info!(
                "Observed {:?} {:?}, {:?}",
                event,
                path.to_str(),
                path.extension()
            );

            if event == FileEvent::Changed || event == FileEvent::Created {
                // PREPROCESSING
                match path.extension().and_then(|ext| ext.to_str()) {
                    Some("vert") | Some("frag") => {
                        self.require_update(AssetKind::ShaderSrc, path);
                        continue;
                    }
                    _ => {}
                }
            }

            match event {
                FileEvent::Created | FileEvent::Changed => {
                    if self.textures.contains_key(&path) {
                        self.require_update(AssetKind::Texture, path);
                    } else if self.meshes.contains_key(&path) {
                        self.require_update(AssetKind::Mesh, path);
                    } else if self.shaders.contains_key(&path) {
                        self.require_update(AssetKind::Shader, path);
                    } else if self.trees.contains_key(&path) {
                        // INSTANT RELOAD vvv
                        let source = path.parent().unwrap();
                        let config = TreePrefabConfig::from_file(&path).unwrap();
                        let data = TreePrefabData {
                            texture: self.texture(config.texture.resolve(&source)),
                            mesh: self.mesh(config.mesh.resolve(&source)),
                        };
                        let prefab = self.trees.get_mut(&path).unwrap();
                        prefab.update(data);
                    } else if self.props.contains_key(&path) {
                        let source = path.parent().unwrap();
                        let config = PropsAssetConfig::from_file(&path).unwrap();
                        let data = PropsAssetData {
                            texture: self.texture(config.texture.resolve(&source)),
                            mesh: self.mesh(config.mesh.resolve(&source)),
                        };
                        let asset = self.props.get_mut(&path).unwrap();
                        asset.update(data);
                    } else if self.farmlands.contains_key(&path) {
                        let source = path.parent().unwrap();
                        let config = FarmlandPrefabConfig::from_file(&path).unwrap();
                        let data = FarmlandPrefabData {
                            props: config
                                .props
                                .iter()
                                .map(|config| Transform {
                                    position: config
                                        .position
                                        .map(|values| Vec3::from(values))
                                        .unwrap_or(Vec3::ZERO),
                                    rotation: Default::default(),
                                    scale: config
                                        .scale
                                        .map(|values| Vec3::from(values))
                                        .unwrap_or(Vec3::ONE),
                                    entity: self.props(config.asset.resolve(&source)),
                                })
                                .collect(),
                        };
                        let asset = self.farmlands.get_mut(&path).unwrap();
                        asset.update(data);
                    }
                }
                FileEvent::Deleted => {}
            }
        }

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
                AssetPayload::Shader { path, data } => match self.shaders.get_mut(&path) {
                    None => {
                        error!(
                            "Unable to update shader {:?}, not registered",
                            path.to_str()
                        );
                    }
                    Some(shader) => {
                        info!("Update shader {:?}", path.to_str());
                        shader.update(data);
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

    fn observe_file_events(&mut self) -> Vec<(PathBuf, FileEvent)> {
        let mut events = match self.file_events.write() {
            Ok(events) => events,
            Err(error) => {
                error!("Unable to observe file events, {:?}", error);
                return vec![];
            }
        };
        std::mem::replace(&mut *events, HashMap::new())
            .into_iter()
            .collect()
    }
}
