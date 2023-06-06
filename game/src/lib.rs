extern crate alloc;
extern crate core;

use log::{error, info};

use datamap::Storage;
pub use domains::*;
pub use rules::*;
pub use update::*;

use crate::api::ActionError::PlayerFarmerNotFound;
use crate::api::{Action, ActionError, Cheat, Event, FarmerBound};
use crate::assembling::AssemblingDomain;
use crate::building::BuildingDomain;
use crate::inventory::{ContainerId, InventoryDomain};
use crate::landscaping::LandscapingDomain;
use crate::math::{Position, Tile, TileMath};
use crate::model::Activity::Idle;
use crate::model::PlayerId;
use crate::model::UniverseDomain;
use crate::model::{Farmer, Universe};
use crate::model::{Farmland, Player};
use crate::model::{Knowledge, TheodoliteKey};
use crate::physics::{BodyId, PhysicsDomain};
use crate::planting::PlantingDomain;
use crate::raising::RaisingDomain;
use crate::timing::TimingDomain;
use crate::working::WorkingDomain;

mod actions;
pub mod api;
mod cheats;
pub mod collections;
mod data;
mod domains;
mod inspection;
mod instantiation;
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

#[macro_export]
macro_rules! emit {
    () => (Ok(vec![]));
    ($($x:expr),+ $(,)?) => (Ok(vec![$($x.into()),*]))
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
            // TODO: define player spawn place
            let farmland = self.universe.farmlands[0];
            let events = self.create_farmer(player_name, "farmer", farmland, [10.5, 10.5])?;
            Ok(events)
        } else {
            info!("Accepts exist player, reconnect");
            Ok(vec![])
        }
    }

    pub fn create_farmland(&mut self, kind: &str) -> Result<Vec<Event>, ActionError> {
        let kind = self.known.farmlands.find(kind)?;

        let (space, create_space) = self.physics.create_space(&kind.space)?;
        let (soil, create_soil) = self.planting.create_soil(&kind.soil)?;
        let (grid, create_grid) = self.building.create_grid(&kind.grid)?;
        let (land, create_land) = self.landscaping.create_land(&kind.land)?;
        let (calendar, create_calendar) = self.timing.create_calendar(&kind.calendar)?;

        emit![
            create_space(),
            create_soil(),
            create_grid(),
            create_land(),
            create_calendar(),
            self.appear_farmland(kind.id, space, soil, grid, land, calendar)?,
        ]
    }

    pub fn create_farmer(
        &mut self,
        player_name: &str,
        kind: &str,
        farmland: Farmland,
        position: Position,
    ) -> Result<Vec<Event>, ActionError> {
        info!("Accepts new player {player_name}");
        self.players_id += 1;
        let player = PlayerId(self.players_id);
        self.players.push(Player {
            id: player,
            name: player_name.to_string(),
        });

        let farmer_kind = self.known.farmers.find(kind)?;
        let body = self.physics.bodies_sequence.introduce().one(BodyId);
        let body_kind = self.known.bodies.find("farmer")?;
        let create_body = self
            .physics
            .create_body(body, farmland.space, body_kind, position)?;

        let [hands, backpack] = self.inventory.containers_id.introduce().many(ContainerId);
        let hands_kind = self.known.containers.find("<hands>")?;
        let backpack_kind = self.known.containers.find("<backpack>")?;
        let create_hands = self.inventory.add_empty_container(hands, &hands_kind)?;
        let create_backpack = self
            .inventory
            .add_empty_container(backpack, &backpack_kind)?;
        let (tether, create_tether) = self.raising.create_tether()?;

        emit![
            create_body(),
            create_hands(),
            create_backpack(),
            create_tether(),
            self.appear_farmer(farmer_kind.id, player, body, hands, backpack, tether)?,
        ]
    }

    pub fn create_theodolite(
        &mut self,
        key: TheodoliteKey,
        farmland: Farmland,
        tile: Tile,
    ) -> Result<Vec<Event>, ActionError> {
        let position = tile.position();
        let kind = self.known.theodolites.get(key)?;
        let (surveyor, create_surveyor) = self
            .building
            .create_surveyor(farmland.grid, kind.surveyor.clone())?;
        let (barrier, create_barrier) = self.physics.create_barrier(
            farmland.space,
            kind.barrier.clone(),
            position,
            true,
            false,
        )?;
        emit![
            create_surveyor(),
            create_barrier(),
            self.appear_theodolite(kind.id, surveyor, barrier)?,
        ]
    }

    pub fn perform_action(
        &mut self,
        player_name: &str,
        action: Action,
    ) -> Result<Vec<Event>, ActionError> {
        let action_debug = format!("{action:?}");
        match self.perform_action_internal(player_name, action) {
            Ok(events) => Ok(events),
            Err(error) => {
                error!("Player {player_name} action {action_debug} error {error:?}");
                Err(error)
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
            Action::EatFoodFromStack {
                creature,
                stack,
                item,
            } => self.eat_food_from_stack(creature, stack, item)?,
            Action::EatFoodFromHands {
                creature,
                farmer,
                item,
            } => self.eat_food_from_hands(creature, farmer, item)?,
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
                    Cheat::SpawnLama { tile } => self.cheat_spawn_lama(farmer, farmland, tile)?,
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
                    FarmerBound::TieCreature { creature } => self.tie_creature(farmer, creature)?,
                    FarmerBound::UntieCreature { creature } => {
                        self.untie_creature(farmer, creature)?
                    }
                    FarmerBound::TieCreature2 { tether, creature } => {
                        self.tie_creature2(farmer, tether, creature)?
                    }
                    FarmerBound::UntieCreature2 { tether, creature } => {
                        self.untie_creature2(farmer, tether, creature)?
                    }
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
                    FarmerBound::UseTheodolite { theodolite } => {
                        self.use_theodolite(farmer, theodolite)?
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

    fn cancel_activity(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let events = self.universe.change_activity(farmer, Idle);
        Ok(occur![events,])
    }

    pub fn load_game_full(&mut self) {
        self.load_game_knowledge().unwrap();
        self.load_game_state().unwrap();
    }
}
