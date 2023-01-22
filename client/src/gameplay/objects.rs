use crate::engine::animatoro::Machine;
use crate::engine::{FarmerAsset, FarmlandAsset, TreeAsset};

use game::building::{Cell, Grid, Room};
use game::collections::Shared;
use game::math::Collider;
use game::model::{Farmer, FarmerKind, Farmland, FarmlandKind, Tree, TreeKind};
use game::physics::{BarrierId, BarrierKey, BarrierKind};
use glam::{Vec2, Vec3};

pub struct FarmerBehaviour {
    pub farmer: Farmer,
    pub kind: Shared<FarmerKind>,
    pub player: String,
    pub asset: FarmerAsset,
    pub estimated_position: Vec2,
    pub rendering_position: Vec3,
    pub last_sync_position: Vec2,
    pub direction: Vec2,
    pub machine: Machine,
}

impl Collider for FarmerBehaviour {
    fn position(&self) -> [f32; 2] {
        [self.rendering_position.x, self.rendering_position.z]
    }

    fn bounds(&self) -> [f32; 2] {
        [0.5, 0.5]
    }
}

pub struct FarmlandBehaviour {
    pub farmland: Farmland,
    pub kind: Shared<FarmlandKind>,
    pub asset: FarmlandAsset,
    pub map: Vec<Vec<[f32; 2]>>,
    pub cells: Vec<Vec<Cell>>,
    pub rooms: Vec<Room>,
}

pub struct TreeBehaviour {
    pub tree: Tree,
    pub kind: Shared<TreeKind>,
    pub asset: TreeAsset,
    pub position: Vec3,
    pub direction: Vec2,
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
