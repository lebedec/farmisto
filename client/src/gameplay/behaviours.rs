use crate::engine::animatoro::Machine;
use crate::engine::{FarmerAsset, FarmlandAsset, TreeAsset};

use game::building::{Cell, Grid, Room};
use game::collections::Shared;
use game::math::Collider;
use game::model::{
    Construction, Drop, Farmer, FarmerKind, Farmland, FarmlandKind, ItemView, Theodolite, Tree,
    TreeKind,
};
use game::physics::{BarrierId, BarrierKey, BarrierKind};
use glam::{Vec2, Vec3};

pub struct FarmerRep {
    pub entity: Farmer,
    pub kind: Shared<FarmerKind>,
    pub player: String,
    pub asset: FarmerAsset,
    pub estimated_position: [f32; 2],
    pub rendering_position: [f32; 2],
    pub last_sync_position: [f32; 2],
    pub direction: [f32; 2],
    pub machine: Machine,
}

impl Collider for FarmerRep {
    fn position(&self) -> [f32; 2] {
        self.rendering_position
    }

    fn bounds(&self) -> [f32; 2] {
        [0.5, 0.5]
    }
}

pub struct FarmlandRep {
    pub farmland: Farmland,
    pub kind: Shared<FarmlandKind>,
    pub asset: FarmlandAsset,
    pub map: Vec<Vec<[f32; 2]>>,
    pub cells: Vec<Vec<Cell>>,
    pub rooms: Vec<Room>,
}

pub struct TreeRep {
    pub tree: Tree,
    pub kind: Shared<TreeKind>,
    pub asset: TreeAsset,
    pub position: [f32; 2],
    pub direction: [f32; 2],
}

pub struct BarrierHint {
    pub id: BarrierId,
    pub kind: BarrierKey,
    pub position: [f32; 2],
    pub bounds: [f32; 2],
}

impl Collider for BarrierHint {
    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn bounds(&self) -> [f32; 2] {
        self.bounds
    }
}

pub struct DropRep {
    pub entity: Drop,
    pub position: [f32; 2],
}

pub struct ConstructionRep {
    pub entity: Construction,
    pub position: [f32; 2],
}

pub struct TheodoliteRep {
    pub entity: Theodolite,
    pub position: [f32; 2],
}
