use game::assembling::Rotation;
use log::error;
use rusty_spine::Skin;
use std::collections::HashMap;

use game::building::{Cell, Room};
use game::collections::Shared;
use game::inventory::{ContainerId, ItemId, ItemKind};
use game::landscaping::LandMap;
use game::math::{Collider, Tile, VectorMath};
use game::model::{
    Activity, Assembly, Cementer, CementerKind, Construction, Creature, CreatureKind, Crop, Door,
    Equipment, Farmer, FarmerKind, Farmland, FarmlandKind, Rest, Stack, Tree, TreeKind,
};
use game::physics::BodyKind;

use crate::assets::{BuildingMaterialAsset, CreatureAsset};
use crate::assets::{CementerAsset, FarmerAsset};
use crate::assets::{CropAsset, DoorAsset};
use crate::assets::{FarmlandAsset, ItemAsset};
use crate::assets::{RestAsset, TreeAsset};
use crate::engine::rendering::{SpineRenderController, TilemapController};

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
    pub moisture: Box<LandMap>,
    pub moisture_capacity: Box<LandMap>,
    pub surface: Box<LandMap>,
    pub surface_tilemap: TilemapController,
    pub cells: Vec<Vec<Cell>>,
    pub rooms: Vec<Room>,
    pub holes: Vec<Vec<u8>>,
    pub construction: BuildingRep,
    pub reconstruction: BuildingRep,
    pub deconstruction: BuildingRep,
    pub buildings: HashMap<u8, BuildingRep>,
    pub season: u8,
    pub season_day: f32,
    pub times_of_day: f32,
}

pub struct BuildingRep {
    pub asset: BuildingMaterialAsset,
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

pub struct StackRep {
    pub entity: Stack,
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

pub enum AssemblyTargetAsset {
    Door {
        door: DoorAsset,
    },
    Cementer {
        cementer: CementerAsset,
        kind: Shared<CementerKind>,
    },
    Rest {
        rest: RestAsset,
    },
}

pub struct AssemblyRep {
    pub entity: Assembly,
    pub asset: AssemblyTargetAsset,
    pub rotation: Rotation,
    pub pivot: Tile,
    pub valid: bool,
}

pub struct DoorRep {
    pub entity: Door,
    pub asset: DoorAsset,
    pub open: bool,
    pub rotation: Rotation,
    pub position: [f32; 2],
}

pub struct RestRep {
    pub entity: Rest,
    pub asset: RestAsset,
    pub rotation: Rotation,
    pub position: [f32; 2],
}

pub struct CementerRep {
    pub entity: Cementer,
    pub kind: Shared<CementerKind>,
    pub asset: CementerAsset,
    pub rotation: Rotation,
    pub position: [f32; 2],
    pub enabled: bool,
    pub broken: bool,
    pub input: bool,
    pub output: bool,
    pub deprecation: f32,
    pub progress: f32,
}

pub struct ItemRep {
    pub id: ItemId,
    pub kind: Shared<ItemKind>,
    pub asset: ItemAsset,
    pub container: ContainerId,
    pub quantity: u8,
}

pub struct CropRep {
    pub entity: Crop,
    pub asset: CropAsset,
    pub spines: Vec<SpineRenderController>,
    pub spine: usize,
    pub position: [f32; 2],
    pub impact: f32,
    pub thirst: f32,
    pub hunger: f32,
    pub growth: f32,
    pub health: f32,
    pub fruits: u8,
}

impl CropRep {
    pub const ANIMATION_TRACK_GROWTH: i32 = 0;

    pub fn synchronize_health(&mut self, health: f32) {
        self.health = health;
    }

    pub fn synchronize_impact(&mut self, impact: f32) {
        self.impact = impact;
    }

    pub fn synchronize_thirst(&mut self, thirst: f32) {
        self.thirst = thirst;
    }

    pub fn synchronize_growth(&mut self, growth: f32) {
        self.growth = growth;
    }

    pub fn synchronize_hunger(&mut self, hunger: f32) {
        self.hunger = hunger;
    }

    pub fn animate_growth(&mut self, _time: f32) {
        self.spine = self.growth.floor() as usize;
    }

    pub fn synchronize_fruits(&mut self, fruits: u8) {
        self.fruits = fruits;
        let ripening = &mut self.spines[3];
        let skins = ripening.skeleton.skeleton.data();
        let skin_names = ["fruit-a", "fruit-b", "fruit-c"];
        let mut skin = Skin::new(&format!("fruits-{}", fruits));
        for name in &skin_names[0..fruits as usize] {
            // TODO: validate skins on load
            let fruit = skins.find_skin(name).unwrap();
            skin.add_skin(&fruit);
        }
        ripening.skeleton.skeleton.set_skin(&skin);
    }

    pub fn is_harvest_phase(&self) -> bool {
        self.growth >= 3.0 && self.growth < 4.0
    }
}

pub struct CreatureRep {
    pub entity: Creature,
    pub asset: CreatureAsset,
    pub kind: Shared<CreatureKind>,
    pub health: f32,
    pub estimated_position: [f32; 2],
    pub rendering_position: [f32; 2],
    pub last_sync_position: [f32; 2],
    pub spine: SpineRenderController,
    pub direction: [f32; 2],
    pub velocity: [f32; 2],
}

impl CreatureRep {
    pub const ANIMATION_TRACK_IDLE: i32 = 0;
    pub const ANIMATION_TRACK_WALK: i32 = 1;
    pub const ANIMATION_TRACK_EAT: i32 = 2;

    pub fn play_eat(&mut self) {
        self.spine
            .skeleton
            .animation_state
            .clear_track(CreatureRep::ANIMATION_TRACK_IDLE);
        self.spine
            .skeleton
            .animation_state
            .add_animation_by_name(CreatureRep::ANIMATION_TRACK_IDLE, "eat", false, 0.0)
            .unwrap();
        self.spine
            .skeleton
            .animation_state
            .add_animation_by_name(CreatureRep::ANIMATION_TRACK_IDLE, "idle", true, 0.0)
            .unwrap();
    }

    pub fn synchronize_position(&mut self, position: [f32; 2]) {
        self.last_sync_position = position;
        let error = position.distance(self.estimated_position);
        if error > 0.5 {
            error!(
                "Correct creature {:?} position error {} {:?} -> {:?}",
                self.entity, error, self.estimated_position, position
            );
            self.estimated_position = position;
            self.rendering_position = position;
        }
    }

    pub fn animate_position(&mut self, time: f32) {
        // smooth movement
        let distance = self.estimated_position.distance(self.last_sync_position);
        let direction = self
            .estimated_position
            .direction_to(self.last_sync_position);
        let translation = self.kind.body.speed * time;
        let estimated_position = if distance < translation {
            self.last_sync_position
        } else {
            self.direction = direction;
            self.estimated_position.add(direction.mul(translation))
        };
        self.estimated_position = estimated_position;
        self.rendering_position = estimated_position;
        self.velocity = direction.mul(translation);
    }
}
