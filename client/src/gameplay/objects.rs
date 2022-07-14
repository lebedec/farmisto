use crate::engine::{FarmerAsset, FarmlandAsset, TreeAsset};
use datamap::{Known, Shared, Storage};
use game::model::{FarmerId, FarmerKind, FarmlandId, FarmlandKind, TreeId, TreeKind};
use glam::{Vec2, Vec3};

pub struct FarmerBehaviour {
    pub id: FarmerId,
    pub kind: Shared<FarmerKind>,
    pub player: String,
    pub asset: FarmerAsset,
    pub estimated_position: Vec2,
    pub rendering_position: Vec3,
    pub last_sync_position: Vec2,
    pub direction: Vec2,
}

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
    pub direction: Vec2,
}

pub struct KnowledgeBase {
    storage: Storage,
    pub trees: Known<TreeKind>,
    pub farmlands: Known<FarmlandKind>,
    pub farmers: Known<FarmerKind>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        let storage = Storage::open("./assets/database.sqlite").unwrap();
        Self {
            storage,
            trees: Default::default(),
            farmlands: Default::default(),
            farmers: Default::default(),
        }
    }

    pub fn reload(&mut self) {
        let storage = &self.storage;
        self.trees.load(storage);
        self.farmlands.load(storage);
        self.farmers.load(storage);
    }
}
