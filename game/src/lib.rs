extern crate alloc;

use datamap::Storage;
pub use domains::*;
pub use rules::*;
pub use update::*;

use crate::api::ActionError::{PlayerFarmerNotFound, Timing};
use crate::api::{Action, ActionError, ActionResponse, Event, FarmerBound};
use crate::assembling::{AssemblingDomain, PlacementId};
use crate::building::{BuildingDomain, Structure, SurveyorId};
use crate::inventory::{ContainerId, FunctionsQuery, InventoryDomain, InventoryError, ItemId};
use crate::landscaping::LandscapingDomain;
use crate::model::Activity::Idle;
use crate::model::Knowledge;
use crate::model::{
    Activity, Assembly, AssemblyKey, Cementer, CementerKey, Creature, CreatureKey, Crop, CropKey,
    Door, DoorKey, Stack,
};
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
pub mod collections;
mod data;
mod domains;
pub mod math;
pub mod model;
mod rules;
mod update;
mod view;

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
            players: vec![],
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
            Action::MoveCreature {
                creature,
                destination,
            } => self.move_creature(creature, destination)?,
            Action::TakeNap { creature } => self.take_nap(creature)?,
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
                    FarmerBound::WaterCrop { crop } => self.water_crop(farmer, crop)?,
                    FarmerBound::HarvestCrop { crop } => self.harvest_crop(farmer, crop)?,
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
                    FarmerBound::RepairDevice { device } => {
                        self.repair_generic_device(farmer, device)?
                    }
                    FarmerBound::ToggleDevice { device } => {
                        self.toggle_generic_device(farmer, device)?
                    }
                    FarmerBound::DigPlace { place } => self.dig_place(farmer, farmland, place)?,
                    FarmerBound::FillBasin { place } => self.fill_basin(farmer, farmland, place)?,
                    FarmerBound::PourWater { place } => self.pour_water(farmer, farmland, place)?,
                    FarmerBound::Relax { rest } => self.relax(farmer, farmland, rest)?,
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

    fn harvest_crop(&mut self, farmer: Farmer, crop: Crop) -> Result<Vec<Event>, ActionError> {
        let crop_kind = self.known.crops.get(crop.key).unwrap();
        let item_kind = &crop_kind.fruits;
        let (new_harvest, capacity) = match self.inventory.get_container_item(farmer.hands) {
            Ok(item) => {
                let kind = item.kind.functions.as_product()?;
                if crop.key != CropKey(kind) {
                    return Err(InventoryError::ItemFunctionNotFound.into());
                }
                (false, item.kind.max_quantity - item.quantity)
            }
            _ => (true, item_kind.max_quantity),
        };
        let (fruits, harvest) = self.planting.harvest_plant(crop.plant, capacity)?;
        let events = if new_harvest {
            let item = self.inventory.items_id.introduce().one(ItemId);
            let create_item = self
                .inventory
                .create_item(item, item_kind, farmer.hands, fruits)?;
            let change_activity = self.universe.change_activity(farmer, Activity::Usage);
            occur![harvest(), create_item(), change_activity,]
        } else {
            let increase_item = self.inventory.increase_item(farmer.hands, fruits)?;

            occur![harvest(), increase_item(),]
        };
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
    ) -> Universe {
        self.universe.crops_id += 1;
        let entity = Crop {
            id: self.universe.crops_id,
            key,
            plant,
            barrier,
            sensor,
        };
        self.universe.crops.push(entity);
        self.look_at_crop(entity)
    }

    pub fn appear_creature(
        &mut self,
        key: CreatureKey,
        body: BodyId,
        animal: AnimalId,
    ) -> Universe {
        self.universe.creatures_id += 1;
        let entity = Creature {
            id: self.universe.creatures_id,
            key,
            body,
            animal,
        };
        self.universe.creatures.push(entity);
        self.look_at_creature(entity)
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
        let look_event = self.look_at_assembly(assembly);
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
        self.look_at_cementer(entity)
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
