use crate::engine::{FarmlandAsset, TreeAsset};
use datamap::{Known, Shared, Storage};
use game::model::{FarmlandId, FarmlandKind, TreeId, TreeKind};
use glam::Vec3;

pub struct FarmlandBehaviour {
    pub id: FarmlandId,
    pub kind: Shared<FarmlandKind>,
    pub asset: FarmlandAsset,
}

pub struct TreeBehaviour {
    pub id: TreeId,
    pub kind: Shared<TreeKind>,
    pub asset: TreeAsset,
    pub position: Vec3,
}

pub struct KnowledgeBase {
    storage: Storage,
    pub trees: Known<TreeKind>,
    pub farmlands: Known<FarmlandKind>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        let storage = Storage::open("./assets/database.sqlite").unwrap();
        Self {
            storage,
            trees: Default::default(),
            farmlands: Default::default(),
        }
    }

    pub fn reload(&mut self) {
        let storage = &self.storage;
        self.trees.load(storage);
        self.farmlands.load(storage);
    }
}
