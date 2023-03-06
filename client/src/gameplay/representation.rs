use crate::engine::{CropAsset, FarmerAsset, FarmlandAsset, SpineAsset, TreeAsset};

use crate::engine::sprites::{SpineSpriteController, TilemapController};
use game::building::{Cell, Room};
use game::collections::Shared;
use game::math::{Collider, VectorMath};
use game::model::{
    Activity, Construction, Crop, Drop, Equipment, Farmer, FarmerKind, Farmland, FarmlandKind,
    Tree, TreeKind,
};
use game::physics::{BarrierId, BodyKind};
use log::error;

pub struct FarmerRep {
    pub entity: Farmer,
    pub kind: Shared<FarmerKind>,
    pub body: Shared<BodyKind>,
    pub player: String,
    pub is_controlled: bool,
    pub asset: FarmerAsset,
    pub estimated_position: [f32; 2],
    pub rendering_position: [f32; 2],
    pub last_sync_position: [f32; 2],
    pub activity: Activity,
}

impl FarmerRep {
    pub fn synchronize_position(&mut self, position: [f32; 2]) {
        self.last_sync_position = position;
        let error = position.distance(self.estimated_position);
        if error > 0.5 {
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
            let translation = self.body.speed * time;
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
        [self.body.radius, self.body.radius]
    }
}

pub struct FarmlandRep {
    pub entity: Farmland,
    pub kind: Shared<FarmlandKind>,
    pub asset: FarmlandAsset,
    pub map: Vec<Vec<[f32; 2]>>,
    pub cells: Vec<Vec<Cell>>,
    pub rooms: Vec<Room>,
    pub holes: Vec<Vec<u8>>,
    pub floor: TilemapController,
    pub roof: TilemapController,
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
    pub tile: [usize; 2],
}

pub struct EquipmentRep {
    pub entity: Equipment,
    pub position: [f32; 2],
}

pub struct CropRep {
    pub entity: Crop,
    pub asset: CropAsset,
    pub spines: Vec<SpineSpriteController>,
    pub spine: usize,
    pub position: [f32; 2],
    pub impact: f32,
    pub thirst: f32,
    pub growth: f32,
}

impl CropRep {
    pub const ANIMATION_TRACK_GROWTH: i32 = 0;

    pub fn synchronize_impact(&mut self, impact: f32) {
        self.impact = impact;
    }

    pub fn synchronize_thirst(&mut self, thirst: f32) {
        self.thirst = thirst;
    }

    pub fn animate_growth(&mut self, time: f32) {
        // let seconds_per_grow_phase = 1.0 / 360.0; // 6 minutes
        let seconds_per_grow_phase = 1.0 / 60.0; // 6 minutes
        self.growth += time * seconds_per_grow_phase;
        if self.growth > 5.0 {
            self.growth -= 5.0;
        }
        self.spine = self.growth.floor() as usize;
    }
}
