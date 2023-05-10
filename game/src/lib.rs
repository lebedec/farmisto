extern crate alloc;
extern crate core;

use datamap::Storage;
pub use domains::*;
use log::info;
pub use rules::*;
pub use update::*;

use crate::api::ActionError::{PlayerFarmerNotFound, Timing};
use crate::api::{Action, ActionError, ActionResponse, Cheat, Event, FarmerBound};
use crate::assembling::{AssemblingDomain, PlacementId};
use crate::building::{BuildingDomain, GridId, Marker, Structure, SurveyorId};
use crate::inventory::{ContainerId, FunctionsQuery, InventoryDomain, InventoryError, ItemId};
use crate::landscaping::LandscapingDomain;
use crate::model::Activity::Idle;
use crate::model::{
    Activity, Assembly, AssemblyKey, Cementer, CementerKey, Construction, Corpse, CorpseKey,
    Creature, CreatureKey, Crop, CropKey, Door, DoorKey, FarmerKey, PlayerId, Stack,
};
use crate::model::{Composter, ComposterKey, Knowledge};
use crate::model::{Equipment, Rest, RestKey};
use crate::model::{EquipmentKey, UniverseDomain};
use crate::model::{Farmer, Universe};
use crate::model::{Player, Purpose};
use crate::physics::{BarrierId, BodyId, PhysicsDomain, SensorId};
use crate::planting::{PlantId, PlantingDomain};
use crate::raising::{AnimalId, RaisingDomain};
use crate::timing::TimingDomain;
use crate::working::{DeviceId, WorkingDomain};

mod actions;
pub mod api;
mod cheats;
pub mod collections;
mod data;
mod domains;
mod inspection;
pub mod math;
pub mod model;
mod rules;
mod update;

#[macro_export]
macro_rules! occur {
    () => (
        vec![]
    );
    ($($x:expr,)*) => (vec![$($x.into()),*])
}

pub struct Game {
    pub known: Knowledge,
    pub timing: TimingDomain,
    pub universe: UniverseDomain,
    pub physics: PhysicsDomain,
    pub planting: PlantingDomain,
    pub landscaping: LandscapingDomain,
    pub building: BuildingDomain,
    pub inventory: InventoryDomain,
    pub raising: RaisingDomain,
    pub assembling: AssemblingDomain,
    pub working: WorkingDomain,
    storage: Storage,
    pub players_id: usize,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new(storage: Storage) -> Self {
        Self {
            known: Knowledge::default(),
            timing: TimingDomain::default(),
            universe: UniverseDomain::default(),
            physics: PhysicsDomain::default(),
            planting: PlantingDomain::default(),
            landscaping: Default::default(),
            building: BuildingDomain::default(),
            inventory: InventoryDomain::default(),
            raising: RaisingDomain::default(),
            assembling: AssemblingDomain::default(),
            working: WorkingDomain::default(),
            storage,
            players_id: 0,
            players: vec![],
        }
    }

    pub fn accept_player(&mut self, player_name: &str) -> Result<Vec<Event>, ActionError> {
        if !self.players.iter().any(|player| player.name == player_name) {
            if player_name == "<AI>" {
                info!("Accepts <AI> player");
                return Ok(vec![]);
            }

            info!("Accepts new player {player_name}");
            self.players_id += 1;
            let player = PlayerId(self.players_id);
            self.players.push(Player {
                id: player,
                name: player_name.to_string(),
            });
            let farmer_kind = self.known.farmers.find("farmer")?;

            // TODO: define player spawn place
            let spawn = [10.5, 10.5];
            let farmland = &self.universe.farmlands[0];

            let body = self.physics.bodies_sequence.introduce().one(BodyId);
            let body_kind = self.known.bodies.find("farmer")?;
            let create_body = self
                .physics
                .create_body(body, farmland.space, body_kind, spawn)?;

            let [hands, backpack] = self.inventory.containers_id.introduce().many(ContainerId);
            let hands_kind = self.known.containers.find("<hands>")?;
            let backpack_kind = self.known.containers.find("<backpack>")?;
            let create_hands = self.inventory.add_empty_container(hands, &hands_kind)?;
            let create_backpack = self
                .inventory
                .add_empty_container(backpack, &backpack_kind)?;

            let events = occur![
                create_body(),
                create_hands(),
                create_backpack(),
                self.appear_farmer(farmer_kind.id, player, body, hands, backpack)?,
            ];
            Ok(events)
        } else {
            info!("Accepts exist player, reconnect");
            Ok(vec![])
        }
    }

    pub fn perform_action(
        &mut self,
        player_name: &str,
        action: Action,
    ) -> Result<Vec<Event>, ActionResponse> {
        match self.perform_action_internal(player_name, action) {
            Ok(events) => Ok(events),
            Err(error) => {
                let player = self
                    .players
                    .iter()
                    .find(|player| &player.name == player_name)
                    .unwrap()
                    .id;
                let farmer = self
                    .universe
                    .farmers
                    .iter()
                    .find(|farmer| farmer.player == player)
                    .unwrap();

                Err(ActionResponse {
                    error,
                    farmer: *farmer,
                    correction: self.universe.get_farmer_activity(*farmer).unwrap(),
                })
            }
        }
    }

    pub fn perform_action_internal(
        &mut self,
        player_name: &str,
        action: Action,
    ) -> Result<Vec<Event>, ActionError> {
        let events = match action {
            Action::EatCrop { creature, crop } => self.eat_crop(creature, crop)?,
            Action::EatFood { creature, item } => self.eat_food(creature, item)?,
            Action::MoveCreature {
                creature,
                destination,
            } => self.move_creature(creature, destination)?,
            Action::TakeNap { creature } => self.take_nap(creature)?,
            Action::Cheat { action } => {
                let player = self
                    .players
                    .iter()
                    .find(|player| &player.name == player_name)
                    .unwrap()
                    .id;
                let farmer = *self
                    .universe
                    .farmers
                    .iter()
                    .find(|farmer| farmer.player == player)
                    .ok_or(PlayerFarmerNotFound(player_name.to_string()))?;
                let farmland = self.universe.farmlands[0];

                match action {
                    Cheat::GrowthUpCrops { growth, radius } => {
                        self.cheat_growth_up_crops(farmer, farmland, growth, radius)?
                    }
                    Cheat::SetCreaturesHealth { health, radius } => {
                        self.cheat_set_creatures_health(farmer, farmland, health, radius)?
                    }
                }
            }
            Action::Farmer { action } => {
                let player = self
                    .players
                    .iter()
                    .find(|player| &player.name == player_name)
                    .unwrap()
                    .id;
                let farmer = *self
                    .universe
                    .farmers
                    .iter()
                    .find(|farmer| farmer.player == player)
                    .ok_or(PlayerFarmerNotFound(player_name.to_string()))?;
                let farmland = self.universe.farmlands[0];

                match action {
                    FarmerBound::Move { destination } => self.move_farmer(farmer, destination)?,
                    FarmerBound::Survey {
                        surveyor,
                        tile,
                        marker,
                    } => self.survey(farmer, farmland, surveyor, tile, marker)?,
                    FarmerBound::Build { construction } => {
                        self.build(farmer, farmland, construction)?
                    }
                    FarmerBound::RemoveConstruction { construction } => {
                        self.remove_construction(farmer, farmland, construction)?
                    }
                    FarmerBound::TakeItemFromStack { stack } => {
                        self.take_item_from_stack(farmer, farmland, stack)?
                    }
                    FarmerBound::TakeItemFromConstruction { construction } => {
                        self.take_item_from_construction(farmer, farmland, construction)?
                    }
                    FarmerBound::TakeItemFromCementer {
                        cementer,
                        container,
                    } => self.take_item_from_cementer(farmer, farmland, cementer, container)?,
                    FarmerBound::TakeItemFromComposter {
                        composter,
                        container,
                    } => self.take_item_from_composter(farmer, farmland, composter, container)?,
                    FarmerBound::PutItemIntoStack { stack } => {
                        self.put_item_into_stack(farmer, farmland, stack)?
                    }
                    FarmerBound::PutItemIntoConstruction { construction } => {
                        self.put_item_into_construction(farmer, farmland, construction)?
                    }
                    FarmerBound::PutItemIntoCementer {
                        cementer,
                        container,
                    } => self.put_item_into_cementer(farmer, farmland, cementer, container)?,
                    FarmerBound::PutItemIntoComposter {
                        composter,
                        container,
                    } => self.put_item_into_composter(farmer, farmland, composter, container)?,
                    FarmerBound::DropItem { tile } => self.drop_item(farmland, farmer, tile)?,
                    FarmerBound::ToggleBackpack => self.toggle_backpack(farmer)?,
                    FarmerBound::Uninstall { equipment } => {
                        self.uninstall_equipment(farmer, farmland, equipment)?
                    }
                    FarmerBound::Install { tile } => {
                        self.install_equipment(farmer, farmland, tile)?
                    }
                    FarmerBound::UseEquipment { equipment } => {
                        self.use_equipment(farmer, equipment)?
                    }
                    FarmerBound::CancelActivity => self.cancel_activity(farmer)?,
                    FarmerBound::ToggleSurveyingOption { option } => {
                        self.toggle_surveying_option(farmer, option)?
                    }
                    FarmerBound::PlantCrop { tile } => self.plant_crop(farmer, farmland, tile)?,
                    FarmerBound::DigUpCrop { crop } => self.dig_up_crop(farmer, farmland, crop)?,
                    FarmerBound::WaterCrop { crop } => self.water_crop(farmer, crop)?,
                    FarmerBound::HarvestCrop { crop } => {
                        self.harvest_crop(farmer, farmland, crop)?
                    }
                    FarmerBound::StartAssembly {
                        pivot: tile,
                        rotation,
                    } => self.start_assembly(farmer, farmland, tile, rotation)?,
                    FarmerBound::MoveAssembly { pivot, rotation } => {
                        self.move_assembly(farmer, farmland, pivot, rotation)?
                    }
                    FarmerBound::FinishAssembly { .. } => self.finish_assembly(farmer, farmland)?,
                    FarmerBound::CancelAssembly => self.cancel_assembly(farmer, farmland)?,
                    FarmerBound::ToggleDoor { door } => self.toggle_door(farmer, door)?,
                    FarmerBound::DisassembleDoor { door } => self.disassemble_door(farmer, door)?,
                    FarmerBound::DisassembleRest { rest } => self.disassemble_rest(farmer, rest)?,
                    FarmerBound::DisassembleCementer { cementer } => {
                        self.disassemble_cementer(farmer, cementer)?
                    }
                    FarmerBound::DisassembleComposter { composter } => {
                        self.disassemble_composter(farmer, composter)?
                    }
                    FarmerBound::RepairDevice { device } => {
                        self.repair_generic_device(farmer, device)?
                    }
                    FarmerBound::ToggleDevice { device } => {
                        self.toggle_generic_device(farmer, device)?
                    }
                    FarmerBound::DigPlace { place } => self.dig_place(farmer, farmland, place)?,
                    FarmerBound::FillBasin { place } => self.fill_basin(farmer, farmland, place)?,
                    FarmerBound::PourWater { place } => self.pour_water(farmer, farmland, place)?,
                    FarmerBound::Fertilize { tile } => self.fertilize(farmer, farmland, tile)?,
                    FarmerBound::Relax { rest } => self.relax(farmer, farmland, rest)?,
                    FarmerBound::CollectCorpse { corpse } => {
                        self.collect_corpse(farmer, farmland, corpse)?
                    }
                }
            }
        };

        Ok(events)
    }

    fn eat_crop(&mut self, creature: Creature, crop: Crop) -> Result<Vec<Event>, ActionError> {
        let bite = 0.3;
        let damage_plant = self.planting.damage_plant(crop.plant, bite)?;
        let feed_animal = self.raising.feed_animal(creature.animal, bite)?;
        let events = occur![
            damage_plant(),
            feed_animal(),
            Universe::CreatureEats { entity: creature },
        ];
        Ok(events)
    }

    fn eat_food(&mut self, creature: Creature, food: ItemId) -> Result<Vec<Event>, ActionError> {
        let feed_animal = self.raising.feed_animal(creature.animal, 0.1)?;
        let events = occur![feed_animal(), Universe::CreatureEats { entity: creature },];
        Ok(events)
    }

    fn move_creature(
        &mut self,
        creature: Creature,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.physics.move_body2(creature.body, destination)?;
        Ok(vec![])
    }

    fn take_nap(&mut self, _creature: Creature) -> Result<Vec<Event>, ActionError> {
        let events = vec![];
        Ok(events)
    }

    fn water_crop(&mut self, _farmer: Farmer, crop: Crop) -> Result<Vec<Event>, ActionError> {
        let water_plant = self.planting.water_plant(crop.plant, 0.5)?;
        let events = occur![water_plant(),];
        Ok(events)
    }

    fn move_farmer(
        &mut self,
        farmer: Farmer,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.physics.move_body2(farmer.body, destination)?;
        Ok(vec![])
    }

    fn cancel_activity(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let events = self.universe.change_activity(farmer, Idle);
        Ok(occur![events,])
    }

    pub fn appear_crop(
        &mut self,
        key: CropKey,
        barrier: BarrierId,
        sensor: SensorId,
        plant: PlantId,
    ) -> Result<Universe, ActionError> {
        self.universe.crops_id += 1;
        let entity = Crop {
            id: self.universe.crops_id,
            key,
            plant,
            barrier,
            sensor,
        };
        self.universe.crops.push(entity);
        self.inspect_crop(entity)
    }

    pub fn appear_farmer(
        &mut self,
        kind: FarmerKey,
        player: PlayerId,
        body: BodyId,
        hands: ContainerId,
        backpack: ContainerId,
    ) -> Result<Universe, ActionError> {
        self.universe.farmers_id += 1;
        let entity = Farmer {
            id: self.universe.farmers_id,
            kind,
            player,
            body,
            hands,
            backpack,
        };
        self.universe
            .farmers_activity
            .insert(entity, Activity::Idle);
        self.universe.farmers.push(entity);
        self.inspect_farmer(entity)
    }

    pub(crate) fn appear_construction(
        &mut self,
        container: ContainerId,
        grid: GridId,
        surveyor: SurveyorId,
        marker: Marker,
        cell: [usize; 2],
    ) -> Vec<Universe> {
        self.universe.constructions_id += 1;
        let construction = Construction {
            id: self.universe.constructions_id,
            container,
            grid,
            surveyor,
            marker,
            cell,
        };
        self.universe.constructions.push(construction);
        vec![Universe::ConstructionAppeared {
            id: construction,
            cell,
        }]
    }

    pub fn appear_creature(
        &mut self,
        key: CreatureKey,
        body: BodyId,
        animal: AnimalId,
    ) -> Result<Universe, ActionError> {
        self.universe.creatures_id += 1;
        let entity = Creature {
            id: self.universe.creatures_id,
            key,
            body,
            animal,
        };
        self.universe.creatures.push(entity);
        self.inspect_creature(entity)
    }

    pub fn appear_corpse(
        &mut self,
        key: CorpseKey,
        barrier: BarrierId,
    ) -> Result<Universe, ActionError> {
        self.universe.corpses_id += 1;
        let entity = Corpse {
            id: self.universe.corpses_id,
            key,
            barrier,
        };
        self.universe.corpses.push(entity);
        self.inspect_corpse(entity)
    }

    pub fn appear_assembling_activity(
        &mut self,
        farmer: Farmer,
        key: AssemblyKey,
        placement: PlacementId,
    ) -> Vec<Universe> {
        self.universe.assembly_id += 1;
        let assembly = Assembly {
            id: self.universe.creatures_id,
            key,
            placement,
        };
        self.universe.assembly.push(assembly);
        let look_event = self.inspect_assembly(assembly);
        let activity = Activity::Assembling { assembly };
        let events = self.universe.change_activity(farmer, activity);
        let mut stream = vec![look_event];
        stream.push(events);
        stream
    }

    pub fn appear_door(
        &mut self,
        key: DoorKey,
        barrier: BarrierId,
        placement: PlacementId,
    ) -> Universe {
        self.universe.doors_id += 1;
        let entity = Door {
            id: self.universe.doors_id,
            key,
            barrier,
            placement,
        };
        self.universe.doors.push(entity);
        self.look_at_door(entity)
    }

    pub fn appear_rest(
        &mut self,
        key: RestKey,
        barrier: BarrierId,
        placement: PlacementId,
    ) -> Universe {
        self.universe.rests_id += 1;
        let entity = Rest {
            id: self.universe.doors_id,
            key,
            barrier,
            placement,
        };
        self.universe.rests.push(entity);
        self.look_at_rest(entity)
    }

    pub fn appear_cementer(
        &mut self,
        key: CementerKey,
        barrier: BarrierId,
        placement: PlacementId,
        input: ContainerId,
        device: DeviceId,
        output: ContainerId,
    ) -> Universe {
        self.universe.cementers_id += 1;
        let entity = Cementer {
            id: self.universe.cementers_id,
            key,
            input,
            device,
            output,
            barrier,
            placement,
        };
        self.universe.cementers.push(entity);
        self.inspect_cementer(entity)
    }

    pub fn appear_composter(
        &mut self,
        key: ComposterKey,
        barrier: BarrierId,
        placement: PlacementId,
        input: ContainerId,
        device: DeviceId,
        output: ContainerId,
    ) -> Universe {
        self.universe.composters_id += 1;
        let entity = Composter {
            id: self.universe.composters_id,
            key,
            input,
            device,
            output,
            barrier,
            placement,
        };
        self.universe.composters.push(entity);
        self.inspect_composter(entity)
    }

    pub fn appear_stack(&mut self, container: ContainerId, barrier: BarrierId) -> Universe {
        self.universe.stacks_id += 1;
        let stack = Stack {
            id: self.universe.stacks_id,
            container,
            barrier,
        };
        self.universe.stacks.push(stack);
        self.look_at_stack(stack)
    }

    pub fn appear_equipment(
        &mut self,
        kind: EquipmentKey,
        purpose: Purpose,
        barrier: BarrierId,
    ) -> Universe {
        self.universe.equipments_id += 1;
        let equipment = Equipment {
            id: self.universe.equipments_id,
            key: kind,
            purpose,
            barrier,
        };
        self.universe.equipments.push(equipment);
        self.look_at_equipment(equipment)
    }

    pub fn load_game_full(&mut self) {
        self.load_game_knowledge().unwrap();
        self.load_game_state().unwrap();
    }
}

#[inline]
fn position_of(tile: [usize; 2]) -> [f32; 2] {
    [tile[0] as f32 + 0.5, tile[1] as f32 + 0.5]
}
