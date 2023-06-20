use game::assembling::Rotation;
use log::error;
use rusty_spine::Skin;
use std::collections::HashMap;

use game::building::{Cell, Marker, Room};
use game::collections::Shared;
use game::inventory::{ContainerId, ItemId, ItemKind};
use game::math::{Collider, Tile, VectorMath};
use game::model::{
    Activity, Assembly, Cementer, CementerKind, Composter, ComposterKind, Construction, Corpse,
    Creature, CreatureKind, Crop, Door, Equipment, EquipmentKind, Farmer, FarmerKind, Farmland,
    FarmlandKind, Rest, Stack, Theodolite, Tree, TreeKind,
};
use game::physics::BodyKind;
use game::raising::{Behaviour, TetherId};

use crate::assets::{
    BuildingMaterialAsset, ComposterAsset, CorpseAsset, CreatureAsset, SpriteAsset,
};
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
    pub fn synchronize_position(&mut self, position: [f32; 2], destination: [f32; 2]) {
        self.last_sync_position = position;
        let error = position.distance(self.estimated_position);
        let skip_obsolete_move_updates =
            self.is_controlled && self.estimated_position.distance(destination) > 0.001;
        if error > 0.5 && !skip_obsolete_move_updates {
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
            // movement interpolation
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
    pub moisture: Vec<f32>,
    pub moisture_capacity: Vec<f32>,
    pub surface: Vec<u8>,
    pub surface_tilemap: TilemapController,
    pub fertility: Vec<f32>,
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
    pub tile: Tile,
    pub marker: Marker,
}

pub struct EquipmentRep {
    pub entity: Equipment,
    pub position: [f32; 2],
    pub kind: Shared<EquipmentKind>,
    pub item: ItemAsset,
}

pub struct TheodoliteRep {
    pub entity: Theodolite,
    pub position: [f32; 2],
    pub mode: u8,
    pub item: ItemAsset,
}

pub enum AssemblyTargetAsset {
    Door {
        door: DoorAsset,
    },
    Cementer {
        cementer: CementerAsset,
        kind: Shared<CementerKind>,
    },
    Composter {
        composter: ComposterAsset,
        kind: Shared<ComposterKind>,
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

pub struct ComposterRep {
    pub entity: Composter,
    pub kind: Shared<ComposterKind>,
    pub asset: ComposterAsset,
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

impl ItemRep {
    pub fn sprite(&self) -> &SpriteAsset {
        if let Some(tileset) = self.asset.quantitative.as_ref() {
            let n = tileset.tiles.len() as f32;
            let f = 1.0 - (self.quantity as f32 / self.kind.max_quantity as f32).min(1.0);
            let i = (f * n - 0.55).round() as usize;
            &tileset.tiles[i]
        } else {
            &self.asset.sprite
        }
    }
}

pub struct CropRep {
    pub entity: Crop,
    pub asset: CropAsset,
    pub spines: Vec<SpineRenderController>,
    pub position: [f32; 2],
    pub impact: f32,
    pub thirst: f32,
    pub hunger: f32,
    pub growth: f32,
    pub health: f32,
    pub fruits: f32,
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

    pub fn spine(&self) -> usize {
        (self.growth.floor() as usize).min(4)
    }

    pub fn synchronize_fruits(&mut self, fruits: f32) {
        self.fruits = fruits;
        let ripening = &mut self.spines[3];
        let skins = ripening.skeleton.skeleton.data();
        let skin_names = ["fruit-a", "fruit-b", "fruit-c"];
        let mut skin = Skin::new(&format!("fruits-{}", fruits));
        for name in &skin_names[0..(fruits as usize).min(skin_names.len())] {
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
    pub age: f32,
    pub weight: f32,
    pub estimated_position: [f32; 2],
    pub rendering_position: [f32; 2],
    pub last_sync_position: [f32; 2],
    pub spine: SpineRenderController,
    pub direction: [f32; 2],
    pub velocity: [f32; 2],
    pub behaviour: Behaviour,
    pub tether: Option<TetherId>,
}

pub fn anim(behaviour: Behaviour) -> &'static str {
    match behaviour {
        Behaviour::Idle => "idle",
        Behaviour::Eating => "eat",
        Behaviour::Sleeping => "sleep",
        Behaviour::Walking => "idle",
    }
}

impl CreatureRep {
    pub const ANIMATION_BASE: i32 = 0;
    pub const ANIMATION_WALK: i32 = 1;
    pub const ANIMATION_AGE: i32 = 3;
    pub const ANIMATION_WEIGHT: i32 = 4;

    pub fn play(&mut self, trigger: Behaviour, behaviour: Behaviour) {
        self.behaviour = behaviour;
        if trigger != behaviour {
            self.once(Self::ANIMATION_BASE, anim(trigger), anim(behaviour));
        } else {
            self.repeat(Self::ANIMATION_BASE, anim(behaviour));
        }
    }

    fn repeat(&mut self, track: i32, animation: &str) {
        self.spine
            .skeleton
            .animation_state
            .set_animation_by_name(track, animation, true)
            .unwrap();
    }

    fn once(&mut self, track: i32, trigger: &str, continuation: &str) {
        self.spine.skeleton.animation_state.clear_track(track);
        self.spine
            .skeleton
            .animation_state
            .add_animation_by_name(track, trigger, false, 0.0)
            .unwrap();
        self.spine
            .skeleton
            .animation_state
            .add_animation_by_name(track, continuation, true, 0.0)
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

pub struct CorpseRep {
    pub entity: Corpse,
    pub asset: CorpseAsset,
    pub position: [f32; 2],
}
