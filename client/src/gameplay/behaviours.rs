use crate::engine::animatoro::Machine;
use crate::engine::{FarmerAsset, FarmlandAsset, TreeAsset};

use game::building::{Cell, Grid, Room};
use game::collections::Shared;
use game::math::{Collider, VectorMath};
use game::model::{
    Construction, Drop, Farmer, FarmerKind, Farmland, FarmlandKind, ItemView, Theodolite, Tree,
    TreeKind,
};
use game::physics::{BarrierId, BarrierKey, BarrierKind};
use glam::{Vec2, Vec3};
use log::error;

pub struct FarmerRep {
    pub entity: Farmer,
    pub kind: Shared<FarmerKind>,
    pub player: String,
    pub is_controlled: bool,
    pub asset: FarmerAsset,
    pub estimated_position: [f32; 2],
    pub rendering_position: [f32; 2],
    pub last_sync_position: [f32; 2],
    pub speed: f32,
    pub direction: [f32; 2],
    pub machine: Machine,
}

impl FarmerRep {
    pub fn synchronize_position(&mut self, position: [f32; 2]) {
        self.last_sync_position = position;
        let error = position.distance(self.estimated_position);
        if error > 64.0 {
            error!(
                "Correct farmer {:?} position error {} {:?} -> {:?}",
                self.entity, error, self.estimated_position, position
            );
            self.estimated_position = position;
            self.rendering_position = position;
        }
    }

    pub fn animate_position(&mut self, time: f32) {
        if self.is_controlled {
            self.rendering_position = self.estimated_position;
        } else {
            // smooth movement
            let distance = self.estimated_position.distance(self.last_sync_position);
            let direction = self
                .estimated_position
                .direction_to(self.last_sync_position);
            let translation = self.speed * time;
            let estimated_position = if distance < translation {
                self.last_sync_position
            } else {
                self.estimated_position.add(direction.mul(translation))
            };
            self.estimated_position = estimated_position;
            self.rendering_position = estimated_position;
        }
    }
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
