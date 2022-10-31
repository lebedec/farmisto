use crate::engine::animatoro::Machine;
use crate::engine::{FarmerAsset, FarmlandAsset, TreeAsset};

use game::collections::Shared;
use game::math::Collider;
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

pub struct BarrierHint {
    pub id: usize,
    pub kind: usize,
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
