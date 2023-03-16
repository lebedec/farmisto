use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{fs, ptr, thread};

use ai::Behaviours;
use ash::{vk, Device};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use rusty_spine::{AnimationStateData, Atlas, SkeletonJson};

use datamap::{Operation, Storage};

use crate::assets::fs::{FileEvent, FileSystem};
use crate::assets::prefabs::{TreeAsset, TreeAssetData};
use crate::assets::tileset::{TilesetAsset, TilesetAssetData, TilesetItem};
use crate::assets::{AssetMap, BehavioursAsset};
use crate::assets::{
    CreatureAsset, CreatureAssetData, CropAsset, CropAssetData, FarmerAsset, FarmerAssetData,
    FarmlandAsset, FarmlandAssetData, FarmlandAssetPropItem, ItemAsset, ItemAssetData,
    PipelineAsset, PipelineAssetData, PropsAsset, PropsAssetData, SamplerAsset, SamplerAssetData,
    ShaderAsset, ShaderAssetData, ShaderCompiler, SpineAsset, SpineAssetData, SpriteAsset,
    SpriteAssetData, TextureAsset, TextureAssetData,
};
use crate::engine::base::Queue;

lazy_static! {
    static ref METRIC_REQUESTS_TOTAL: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!(
            "asset_requests_total",
            "asset_requests_total",
            &["type"]
        )
        .unwrap();
}

pub struct Assets {
    pub storage: Storage,

    loading_requests: Arc<RwLock<Vec<AssetRequest>>>,
    loading_result: Receiver<AssetPayload>,

    file_system: FileSystem,

    textures_default: TextureAssetData,
    textures_white: TextureAsset,
    textures: HashMap<PathBuf, TextureAsset>,

    shaders: HashMap<PathBuf, ShaderAsset>,
    pipelines: HashMap<String, PipelineAsset>,
    sprites: HashMap<String, SpriteAsset>,
    tilesets: HashMap<String, TilesetAsset>,
    samplers: HashMap<String, SamplerAsset>,
    spines: HashMap<String, SpineAsset>,

    behaviours: HashMap<PathBuf, BehavioursAsset>,

    // prefabs
    farmlands: HashMap<String, FarmlandAsset>,
    trees: HashMap<String, TreeAsset>,
    props: HashMap<String, PropsAsset>,
    farmers: HashMap<String, FarmerAsset>,
    items: HashMap<String, ItemAsset>,
    crops: HashMap<String, CropAsset>,
    creatures: HashMap<String, CreatureAsset>,

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
}

#[derive(Debug)]
pub enum AssetKind {
    Texture,
    Shader,
    ShaderSrc,
}

fn spawn_loader(
    loader: i32,
    requests: Arc<RwLock<Vec<AssetRequest>>>,
    queue: Arc<Queue>,
    result: Sender<AssetPayload>,
    device: Device,
    pool: vk::CommandPool,
) {
    thread::Builder::new()
        .name(format!("loader-{}", loader))
        .spawn(move || {
            info!("[loader-{}] Start loader", loader);
            loop {
                let request = { requests.write().unwrap().pop() };
                if let Some(request) = request {
                    debug!(
                        "[loader-{}] Load {:?} {:?}",
                        loader,
                        request.kind,
                        request.path.to_str()
                    );
                    let path = request.path.clone();
                    match request.kind {
                        AssetKind::Texture => {
                            let data = TextureAssetData::create_and_read_image(
                                &device,
                                pool,
                                queue.clone(),
                                path.as_os_str().to_str().unwrap(),
                            );
                            result.send(AssetPayload::Texture { path, data }).unwrap();
                        }
                        AssetKind::Shader => {
                            match ShaderAssetData::from_spirv_file(&queue, &path) {
                                Ok(data) => {
                                    result.send(AssetPayload::Shader { path, data }).unwrap();
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
                            debug!("[loader-{}] Compile shader {:?}", loader, path.to_str());
                            let compiler = ShaderCompiler::new();
                            compiler.compile_file(path);
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(15))
                }
            }
        })
        .unwrap();
}

impl Assets {
    pub fn new(device: Device, pool: vk::CommandPool, queue: Arc<Queue>) -> Self {
        let storage = Storage::open("./assets/assets.sqlite").unwrap();
        storage.setup_tracking().unwrap();

        let textures_default = TextureAssetData::create_and_read_image(
            &device,
            pool,
            queue.clone(),
            "./assets/fallback/texture.png",
        );
        let textures_white = TextureAsset::from(TextureAssetData::create_and_read_image(
            &device,
            pool,
            queue.clone(),
            "./assets/fallback/white.png",
        ));
        let textures = HashMap::new();
        let shaders = HashMap::new();

        let loading_requests = Arc::new(RwLock::new(Vec::<AssetRequest>::new()));
        let (loading, loading_result) = channel();

        let loaders = 4;
        for loader in 0..loaders {
            let loaders_requests = loading_requests.clone();
            let loader_queue = queue.clone();
            let loader_result = loading.clone();
            let loader_device = device.clone();
            spawn_loader(
                loader,
                loaders_requests,
                loader_queue,
                loader_result,
                loader_device,
                pool,
            );
        }

        let file_system = if Self::is_development() {
            // todo: organize pre-processing workers
            info!(
                "Shader compiler version: {}",
                ShaderCompiler::new().version()
            );

            let files = vec!["png", "json", "yaml", "space3", "frag", "vert", "spv"];
            info!("Configures file system watch for {:?}", files);
            FileSystem::watch(files)
        } else {
            info!("Skips file system watch configuration because of production mode");
            FileSystem::idle()
        };

        Self {
            storage,
            textures_default,
            textures_white,
            textures,
            loading_requests,
            loading_result,
            shaders,
            file_system,
            queue,
            farmlands: Default::default(),
            trees: Default::default(),
            props: Default::default(),
            farmers: Default::default(),
            pipelines: Default::default(),
            sprites: Default::default(),
            tilesets: Default::default(),
            samplers: Default::default(),
            spines: Default::default(),
            items: Default::default(),
            crops: Default::default(),
            creatures: Default::default(),
            behaviours: Default::default(),
        }
    }

    pub fn behaviours<P: AsRef<Path>>(&mut self, path: P) -> BehavioursAsset {
        METRIC_REQUESTS_TOTAL
            .with_label_values(&["behaviours"])
            .inc();
        let path = fs::canonicalize(path).unwrap();
        match self.behaviours.get(&path) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_behaviours_data(path.clone());
                let behaviours = BehavioursAsset::from(data);
                self.behaviours.insert(path, behaviours.share());
                behaviours
            }
        }
    }

    pub fn shader<P: AsRef<Path>>(&mut self, path: P) -> ShaderAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["shader"]).inc();
        let path = fs::canonicalize(path).unwrap();
        if let Some(shader) = self.shaders.get(&path) {
            return shader.share();
        }
        let data = ShaderAssetData::from_spirv_file(&self.queue, &path).unwrap();
        let shader = ShaderAsset::from(data);
        self.shaders.insert(path.clone(), shader.share());
        shader
    }

    pub fn texture<P: AsRef<Path>>(&mut self, path: P) -> TextureAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["texture"]).inc();
        let path = fs::canonicalize(path).unwrap();
        if let Some(texture) = self.textures.get(&path) {
            return texture.clone();
        }
        let texture = TextureAsset::from(self.textures_default.clone());
        self.textures.insert(path.clone(), texture.clone());
        self.require_update(AssetKind::Texture, path);
        texture
    }

    pub fn texture_white(&self) -> TextureAsset {
        self.textures_white.clone()
    }

    pub fn spine(&mut self, path: &str) -> SpineAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["spine"]).inc();
        if let Some(asset) = self.spines.get(path) {
            return asset.share();
        }
        // Spine files naming convention: place atlas file in same directory and name as folder
        // TODO: support multiple atlases
        let spine_folder = Path::new(path)
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let atlas_path = Path::new(path).with_file_name(format!("{}.atlas", spine_folder));
        let mut atlas = Atlas::new_from_file(&atlas_path).unwrap();
        let spine_asset_folder = PathBuf::from(atlas_path);
        let spine_asset_folder = spine_asset_folder.parent().unwrap();
        let first_page = atlas.pages_mut().next().unwrap();
        let texture_path = spine_asset_folder.join(first_page.name());
        let texture = self.texture(texture_path);

        let mut skeleton_json = SkeletonJson::new(Arc::new(atlas));
        let s = skeleton_json.read_skeleton_data_file(path);
        let skeleton = Arc::new(s.unwrap());
        let animation = Arc::new(AnimationStateData::new(skeleton.clone()));
        let data = SpineAssetData {
            animation,
            skeleton,
            atlas: texture,
        };
        let asset = SpineAsset::from(data);
        self.spines.insert(path.to_string(), asset.share());
        asset
    }

    pub fn sprite(&mut self, name: &str) -> SpriteAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["sprite"]).inc();
        match self.sprites.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_sprite_data(name).unwrap();
                self.sprites.publish(name, data)
            }
        }
    }

    pub fn tileset(&mut self, name: &str) -> TilesetAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["tileset"]).inc();
        match self.tilesets.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_tileset_data(name).unwrap();
                self.tilesets.publish(name, data)
            }
        }
    }

    pub fn sampler(&mut self, name: &str) -> SamplerAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["sampler"]).inc();
        match self.samplers.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.create_sampler_from_data(name).unwrap();
                self.samplers.publish(name, data)
            }
        }
    }

    pub fn pipeline(&mut self, name: &str) -> PipelineAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["pipeline"]).inc();
        match self.pipelines.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_pipeline_data(name).unwrap();
                self.pipelines.publish(name, data)
            }
        }
    }

    pub fn tree(&mut self, name: &str) -> TreeAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["tree"]).inc();
        match self.trees.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_tree_data(name).unwrap();
                self.trees.publish(name, data)
            }
        }
    }

    pub fn item(&mut self, name: &str) -> ItemAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["item"]).inc();
        match self.items.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_item_data(name).unwrap();
                self.items.publish(name, data)
            }
        }
    }

    pub fn crop(&mut self, name: &str) -> CropAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["crop"]).inc();
        match self.crops.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_crop_data(name).unwrap();
                self.crops.publish(name, data)
            }
        }
    }

    pub fn creature(&mut self, name: &str) -> CreatureAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["creature"]).inc();
        match self.creatures.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_creature_data(name).unwrap();
                self.creatures.publish(name, data)
            }
        }
    }

    pub fn farmland(&mut self, name: &str) -> FarmlandAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["farmland"]).inc();
        match self.farmlands.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_farmland_data(name).unwrap();
                self.farmlands.publish(name, data)
            }
        }
    }

    pub fn farmer(&mut self, name: &str) -> FarmerAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["farmer"]).inc();
        match self.farmers.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_farmer_data(name).unwrap();
                self.farmers.publish(name, data)
            }
        }
    }

    pub fn props(&mut self, name: &str) -> PropsAsset {
        METRIC_REQUESTS_TOTAL.with_label_values(&["props"]).inc();
        match self.props.get(name) {
            Some(asset) => asset.share(),
            None => {
                let data = self.load_props_data(name).unwrap();
                self.props.publish(name, data)
            }
        }
    }

    pub fn load_tree_data(&mut self, id: &str) -> Result<TreeAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<TreeAssetData>(id);
        let texture: String = entry.get("texture")?;
        let data = TreeAssetData {
            texture: self.texture(texture),
        };
        Ok(data)
    }

    pub fn load_item_data(&mut self, id: &str) -> Result<ItemAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<ItemAssetData>(id);
        let sprite: String = entry.get("sprite")?;
        let data = ItemAssetData {
            sprite: self.sprite(&sprite),
        };
        Ok(data)
    }

    pub fn load_crop_data(&mut self, id: &str) -> Result<CropAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<CropAssetData>(id);
        let folder: String = entry.get("spine")?;
        let data = CropAssetData {
            sprout: self.spine(&format!("{}/sprout.json", folder)),
            vegetating: self.spine(&format!("{}/vegetating.json", folder)),
            flowering: self.spine(&format!("{}/flowering.json", folder)),
            ripening: self.spine(&format!("{}/ripening.json", folder)),
            withering: self.spine(&format!("{}/withering.json", folder)),
            damage_mask: self.texture(&format!("{}/damage-mask.png", folder)),
        };
        Ok(data)
    }

    pub fn load_creature_data(&mut self, id: &str) -> Result<CreatureAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<CreatureAssetData>(id);
        let spine: String = entry.get("spine")?;
        let coloration: String = entry.get("coloration")?;
        let data = CreatureAssetData {
            spine: self.spine(&spine),
            coloration: self.texture(coloration),
        };
        Ok(data)
    }

    pub fn load_farmland_data(&mut self, id: &str) -> Result<FarmlandAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<FarmlandAssetData>(id);
        let entries = self.storage.fetch_many::<FarmlandAssetPropItem>(id);
        let mut props = vec![];
        for entry in entries {
            let asset: String = entry.get("asset")?;
            let data = FarmlandAssetPropItem {
                position: entry.get("position")?,
                rotation: entry.get("rotation")?,
                scale: entry.get("scale")?,
                asset: self.props(&asset),
            };
            props.push(data)
        }
        let mut building_templates = vec![];
        let building_template_names: Vec<String> = entry.get("building_templates")?;
        for name in &building_template_names {
            building_templates.push(self.tileset(name));
        }
        let building_template_surveying: String = entry.get("building_template_surveying")?;
        let data = FarmlandAssetData {
            props,
            texture: self.texture(entry.get_string("texture")?),
            sampler: self.sampler(entry.get("sampler")?),
            building_templates,
            building_template_surveying: self.tileset(&building_template_surveying),
        };
        Ok(data)
    }

    pub fn load_farmer_data(&mut self, id: &str) -> Result<FarmerAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<FarmerAssetData>(id);
        let texture: String = entry.get("texture")?;
        let data = FarmerAssetData {
            texture: self.texture(texture),
        };
        Ok(data)
    }

    pub fn load_props_data(&mut self, id: &str) -> Result<PropsAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<PropsAssetData>(id);
        let texture: String = entry.get("texture")?;
        let data = PropsAssetData {
            texture: self.texture(texture),
        };
        Ok(data)
    }

    pub fn load_pipeline_data(&mut self, id: &str) -> Result<PipelineAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<PipelineAssetData>(id);
        let fragment: String = entry.get("fragment")?;
        let vertex: String = entry.get("vertex")?;
        let data = PipelineAssetData {
            name: id.to_string(),
            fragment: self.shader(fragment),
            vertex: self.shader(vertex),
            changed: true,
        };
        Ok(data)
    }

    pub fn load_sprite_data(&mut self, id: &str) -> Result<SpriteAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<SpriteAssetData>(id);
        let texture: String = entry.get("texture")?;
        let data = SpriteAssetData {
            texture: self.texture(texture),
            position: entry.get("position")?,
            size: entry.get("size")?,
            sampler: self.sampler(entry.get("sampler")?),
            pivot: entry.get("pivot")?,
        };
        Ok(data)
    }

    pub fn load_tileset_data(&mut self, id: &str) -> Result<TilesetAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<TilesetAssetData>(id);
        let texture = self.texture(entry.get_string("texture")?);
        let sampler = self.sampler(entry.get("sampler")?);
        let items: Vec<TilesetItem> = entry.get("tiles")?;
        let mut tiles = vec![];
        for item in items {
            tiles.push(SpriteAsset::from(SpriteAssetData {
                texture: texture.clone(),
                position: item.src,
                size: item.size,
                sampler: sampler.share(),
                pivot: item.pivot,
            }))
        }
        let data = TilesetAssetData {
            texture,
            sampler,
            tiles,
        };
        Ok(data)
    }

    pub fn load_behaviours_data(&mut self, path: PathBuf) -> Behaviours {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }

    pub fn create_sampler_from_data(
        &mut self,
        id: &str,
    ) -> Result<SamplerAssetData, serde_json::Error> {
        let entry = self.storage.fetch_one::<SamplerAssetData>(id);
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: match entry.get_string("mag_filter")? {
                "NEAREST" => vk::Filter::NEAREST,
                "LINEAR" => vk::Filter::LINEAR,
                _ => vk::Filter::NEAREST,
            },
            min_filter: match entry.get_string("min_filter")? {
                "NEAREST" => vk::Filter::NEAREST,
                "LINEAR" => vk::Filter::LINEAR,
                _ => vk::Filter::NEAREST,
            },
            mipmap_mode: match entry.get_string("mipmap_mode")? {
                "NEAREST" => vk::SamplerMipmapMode::NEAREST,
                "LINEAR" => vk::SamplerMipmapMode::LINEAR,
                _ => vk::SamplerMipmapMode::NEAREST,
            },
            address_mode_u: match entry.get_string("address_mode_u")? {
                "REPEAT" => vk::SamplerAddressMode::REPEAT,
                "MIRRORED_REPEAT" => vk::SamplerAddressMode::MIRRORED_REPEAT,
                "CLAMP_TO_EDGE" => vk::SamplerAddressMode::CLAMP_TO_EDGE,
                "CLAMP_TO_BORDER" => vk::SamplerAddressMode::CLAMP_TO_BORDER,
                _ => vk::SamplerAddressMode::REPEAT,
            },
            address_mode_v: match entry.get_string("address_mode_v")? {
                "REPEAT" => vk::SamplerAddressMode::REPEAT,
                "MIRRORED_REPEAT" => vk::SamplerAddressMode::MIRRORED_REPEAT,
                "CLAMP_TO_EDGE" => vk::SamplerAddressMode::CLAMP_TO_EDGE,
                "CLAMP_TO_BORDER" => vk::SamplerAddressMode::CLAMP_TO_BORDER,
                _ => vk::SamplerAddressMode::REPEAT,
            },
            address_mode_w: match entry.get_string("address_mode_w")? {
                "REPEAT" => vk::SamplerAddressMode::REPEAT,
                "MIRRORED_REPEAT" => vk::SamplerAddressMode::MIRRORED_REPEAT,
                "CLAMP_TO_EDGE" => vk::SamplerAddressMode::CLAMP_TO_EDGE,
                "CLAMP_TO_BORDER" => vk::SamplerAddressMode::CLAMP_TO_BORDER,
                _ => vk::SamplerAddressMode::REPEAT,
            },
            mip_lod_bias: entry.get("mip_lod_bias")?,
            anisotropy_enable: entry.get("anisotropy_enable")?,
            max_anisotropy: entry.get("max_anisotropy")?,
            compare_enable: entry.get("compare_enable")?,
            compare_op: match entry.get_string("compare_op")? {
                "NEVER" => vk::CompareOp::NEVER,
                "LESS" => vk::CompareOp::LESS,
                "EQUAL" => vk::CompareOp::EQUAL,
                "LESS_OR_EQUAL" => vk::CompareOp::LESS_OR_EQUAL,
                "GREATER" => vk::CompareOp::GREATER,
                "NOT_EQUAL" => vk::CompareOp::NOT_EQUAL,
                "GREATER_OR_EQUAL" => vk::CompareOp::GREATER_OR_EQUAL,
                "ALWAYS" => vk::CompareOp::ALWAYS,
                _ => vk::CompareOp::ALWAYS,
            },
            min_lod: entry.get("min_lod")?,
            max_lod: entry.get("max_lod")?,
            border_color: match entry.get_string("border_color")? {
                "FLOAT_TRANSPARENT_BLACK" => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
                "INT_TRANSPARENT_BLACK" => vk::BorderColor::INT_TRANSPARENT_BLACK,
                "FLOAT_OPAQUE_BLACK" => vk::BorderColor::FLOAT_OPAQUE_BLACK,
                "INT_OPAQUE_BLACK" => vk::BorderColor::INT_OPAQUE_BLACK,
                "FLOAT_OPAQUE_WHITE" => vk::BorderColor::FLOAT_OPAQUE_WHITE,
                "INT_OPAQUE_WHITE" => vk::BorderColor::INT_OPAQUE_WHITE,
                _ => vk::BorderColor::INT_OPAQUE_BLACK,
            },
            unnormalized_coordinates: entry.get("unnormalized_coordinates")?,
        };
        let handle = unsafe {
            self.queue
                .device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        };
        Ok(SamplerAssetData { handle })
    }

    fn require_update(&mut self, kind: AssetKind, path: PathBuf) {
        debug!("Require update {:?} {:?}", kind, path.to_str());
        let mut requests = self.loading_requests.write().unwrap();
        requests.push(AssetRequest { path, kind });
    }

    pub fn process_assets_loading(&mut self) {
        if Self::is_development() {
            self.detect_changes()
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
                        debug!("Update texture {:?}", path.to_str());
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
                        debug!("Update shader {:?}", path.to_str());
                        shader.update(data);
                        for pipeline in self.pipelines.values_mut() {
                            if pipeline.fragment.module == shader.module
                                || pipeline.vertex.module == shader.module
                            {
                                pipeline.changed = true;
                            }
                        }
                    }
                },
            }
        }
    }

    fn reload_dictionaries(&mut self) -> Result<(), rusqlite::Error> {
        let changes = self.storage.track_changes::<String>()?;
        for change in changes {
            match change.operation {
                Operation::Update => match change.entity.as_str() {
                    "FarmerAssetData" => {
                        let data = self.load_farmer_data(&change.id).unwrap();
                        self.farmers.get_mut(&change.id).unwrap().update(data);
                    }
                    "FarmlandAssetData" | "FarmlandAssetPropItem" => {
                        let data = self.load_farmland_data(&change.id).unwrap();
                        self.farmlands.get_mut(&change.id).unwrap().update(data);
                    }
                    "TreeAssetData" => {
                        let data = self.load_tree_data(&change.id).unwrap();
                        self.trees.get_mut(&change.id).unwrap().update(data);
                    }
                    "ItemAssetData" => {
                        let data = self.load_item_data(&change.id).unwrap();
                        self.items.get_mut(&change.id).unwrap().update(data);
                    }
                    "CropAssetData" => {
                        let data = self.load_crop_data(&change.id).unwrap();
                        self.crops.get_mut(&change.id).unwrap().update(data);
                    }
                    "PropsAssetData" => {
                        let data = self.load_props_data(&change.id).unwrap();
                        self.props.get_mut(&change.id).unwrap().update(data);
                    }
                    "PipelineAssetData" => {
                        let data = self.load_pipeline_data(&change.id).unwrap();
                        self.pipelines.get_mut(&change.id).unwrap().update(data);
                    }
                    "SpriteAssetData" => {
                        let data = self.load_sprite_data(&change.id).unwrap();
                        self.sprites.get_mut(&change.id).unwrap().update(data);
                    }
                    "TilesetAssetData" => {
                        let data = self.load_tileset_data(&change.id).unwrap();
                        self.tilesets.get_mut(&change.id).unwrap().update(data);
                    }
                    "SamplerAssetData" => {
                        let data = self.create_sampler_from_data(&change.id).unwrap();
                        self.samplers.get_mut(&change.id).unwrap().update(data);
                    }
                    _ => {
                        error!("Handle of {:?} not implemented yet", change)
                    }
                },
                Operation::Insert => {
                    warn!("Insert of {} not implemented yet", change.entity);
                }
                Operation::Delete => {
                    warn!("Delete of {} not implemented yet", change.entity);
                }
            }
        }
        Ok(())
    }

    pub fn is_development() -> bool {
        std::env::var("DEV_MODE").is_ok()
    }

    pub fn detect_changes(&mut self) {
        if let Err(error) = self.reload_dictionaries() {
            error!("Unable to reload dictionaries, {:?}", error);
        }

        for (path, event) in self.file_system.observe_file_events() {
            debug!(
                "Observed {:?} {:?}, {:?}",
                event,
                path.to_str(),
                path.extension()
            );

            if event == FileEvent::Changed || event == FileEvent::Created {
                // PRE-PROCESSING
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
                    } else if self.shaders.contains_key(&path) {
                        self.require_update(AssetKind::Shader, path);
                    } else if self.behaviours.contains_key(&path) {
                        let data = self.load_behaviours_data(path.clone());
                        let behaviours = self.behaviours.get_mut(&path).unwrap();
                        info!("UPDATE BEH {:?}", path.to_str());
                        behaviours.update(data);
                    }
                }
                FileEvent::Deleted => {}
            }
        }
    }
}
