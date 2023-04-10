extern crate alloc;
extern crate core;

use rand::thread_rng;

use actions::*;
use datamap::Storage;
pub use domains::*;
pub use update::*;

use crate::api::ActionError::PlayerFarmerNotFound;
use crate::api::{Action, ActionError, ActionResponse, Event, FarmerBound};
use crate::assembling::{AssemblingDomain, PlacementId, Rotation};
use crate::building::{BuildingDomain, Marker, Material, Stake, Structure, SurveyorId};
use crate::inventory::{ContainerId, FunctionsQuery, InventoryDomain, InventoryError};
use crate::math::VectorMath;
use crate::model::Activity::Idle;
use crate::model::Equipment;
use crate::model::UniverseError;
use crate::model::{
    Activity, Assembly, AssemblyKey, AssemblyTarget, Cementer, CementerKey, Creature, CreatureKey,
    Crop, CropKey, Door, DoorKey, Stack,
};
use crate::model::{Construction, Farmer, Universe};
use crate::model::{EquipmentKey, PurposeDescription, UniverseDomain};
use crate::model::{Farmland, Knowledge};
use crate::model::{Player, Purpose};
use crate::physics::{BarrierId, BodyId, PhysicsDomain, SensorId};
use crate::planting::{PlantId, PlantingDomain};
use crate::raising::{AnimalId, RaisingDomain};
use crate::working::{DeviceId, Process, Working, WorkingDomain};

mod actions;
pub mod api;
pub mod collections;
mod data;
mod domains;
pub mod math;
pub mod model;
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
    pub universe: UniverseDomain,
    pub physics: PhysicsDomain,
    pub planting: PlantingDomain,
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
            universe: UniverseDomain::default(),
            physics: PhysicsDomain::default(),
            planting: PlantingDomain::default(),
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
                    FarmerBound::TakeItem { container } => self.take_item(farmer, container)?,
                    FarmerBound::PutItem { container } => self.put_item(farmer, container)?,
                    FarmerBound::DropItem { tile } => self.drop_item(farmer, tile)?,
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
                    FarmerBound::DisassembleCementer { cementer } => {
                        self.disassemble_cementer(farmer, cementer)?
                    }
                    FarmerBound::RepairDevice { device } => {
                        self.repair_generic_device(farmer, device)?
                    }
                    FarmerBound::ToggleDevice { device } => {
                        self.toggle_generic_device(farmer, device)?
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
            let (_, create_item) = self
                .inventory
                .create_item(item_kind, farmer.hands, fruits)?;
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

    fn teardown_constructions(
        &mut self,
        _farmer: Farmer,
        _farmland: Farmland,
        surveyor: SurveyorId,
    ) -> Result<Vec<Event>, ActionError> {
        let constructions: Vec<Construction> = self
            .universe
            .constructions
            .iter()
            .filter(|construction| construction.surveyor == surveyor)
            .cloned()
            .collect();

        let containers = constructions
            .iter()
            .map(|construction| construction.container)
            .collect();

        let destroy_containers = self.inventory.destroy_containers(containers, false)?;

        let events = occur![
            destroy_containers(),
            constructions
                .into_iter()
                .map(|id| self.universe.vanish_construction(id))
                .flatten()
                .collect::<Vec<Universe>>(),
        ];

        Ok(events)
    }

    fn cancel_activity(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let events = self.universe.change_activity(farmer, Idle);
        Ok(occur![events,])
    }

    fn toggle_surveying_option(
        &mut self,
        farmer: Farmer,
        option: u8,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        if let Activity::Surveying {
            equipment,
            mut selection,
        } = activity
        {
            selection = option as usize % 4;
            let activity = Activity::Surveying {
                equipment,
                selection,
            };
            let events = self.universe.change_activity(farmer, activity);
            Ok(occur![events,])
        } else {
            // TODO: rework expected activity attribute
            return Err(UniverseError::FarmerActivityMismatch {
                actual: activity,
                expected: Activity::Surveying {
                    equipment: Equipment {
                        id: 0,
                        key: EquipmentKey(0),
                        purpose: Purpose::Surveying {
                            surveyor: SurveyorId(0),
                        },
                        barrier: BarrierId(0),
                    },
                    selection: 0,
                },
            }
            .into());
        }
    }

    fn start_assembly(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        pivot: [usize; 2],
        rotation: Rotation,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let key = item.kind.functions.as_assembly(AssemblyKey)?;
        self.known.assembly.get(key)?;
        let (placement, start_placement) = self.assembling.start_placement(rotation, pivot)?;
        let events = occur![
            start_placement(),
            self.appear_assembling_activity(farmer, key, placement),
        ];
        Ok(events)
    }

    fn move_assembly(
        &mut self,
        farmer: Farmer,
        _farmland: Farmland,
        pivot: [usize; 2],
        rotation: Rotation,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        let assembly = activity.as_assembling()?;
        let update_placement =
            self.assembling
                .update_placement(assembly.placement, rotation, pivot)?;
        let events = occur![update_placement(),];
        Ok(events)
    }

    fn cancel_assembly(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        let assembly = activity.as_assembling()?;
        let cancel_placement = self.assembling.cancel_placement(assembly.placement)?;
        self.universe.change_activity(farmer, Activity::Usage);
        let events = occur![
            cancel_placement(),
            self.universe.vanish_assembly(assembly),
            self.universe.change_activity(farmer, Activity::Usage),
        ];
        Ok(events)
    }

    pub fn toggle_door(&mut self, _farmer: Farmer, door: Door) -> Result<Vec<Event>, ActionError> {
        let barrier = self.physics.get_barrier(door.barrier)?;
        let door_open = Universe::DoorChanged {
            entity: door,
            open: barrier.active,
        };
        let toggle_door = self.physics.change_barrier(barrier.id, !barrier.active)?;
        let events = occur![toggle_door(), door_open,];
        Ok(events)
    }

    fn toggle_backpack(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let backpack_empty = self
            .inventory
            .get_container(farmer.backpack)?
            .items
            .is_empty();
        let hands_empty = self.inventory.get_container(farmer.hands)?.items.is_empty();
        let mut events = vec![];
        if hands_empty && !backpack_empty {
            let transfer = self.inventory.pop_item(farmer.backpack, farmer.hands)?;
            events = occur![
                transfer(),
                self.universe.change_activity(farmer, Activity::Usage),
            ];
        }
        if !hands_empty && backpack_empty {
            let transfer = self.inventory.pop_item(farmer.hands, farmer.backpack)?;
            events = occur![
                transfer(),
                self.universe.change_activity(farmer, Activity::Idle),
            ];
        }
        Ok(events)
    }

    fn survey(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        surveyor: SurveyorId,
        cell: [usize; 2],
        marker: Marker,
    ) -> Result<Vec<Event>, ActionError> {
        let stake = Stake { marker, cell };
        let survey = self.building.survey(surveyor, stake)?;
        let container_kind = self.known.containers.find("<construction>")?;
        let container = self.inventory.containers_id.introduce().one(ContainerId);
        let create_container = self.inventory.add_container(container, &container_kind)?;
        let appearance =
            self.universe
                .appear_construction(container, farmland.grid, surveyor, marker, cell);
        let events = occur![survey(), create_container(), appearance,];
        Ok(events)
    }

    fn remove_construction(
        &mut self,
        _farmer: Farmer,
        _farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let destroy_container = self
            .inventory
            .destroy_containers(vec![construction.container], false)?;
        let destroy_marker = self
            .building
            .unmark(construction.surveyor, construction.cell)?;
        let events = occur![
            destroy_container(),
            destroy_marker(),
            self.universe.vanish_construction(construction),
        ];
        Ok(events)
    }

    fn build(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        match construction.marker {
            Marker::Construction(_) => {
                let item = self.inventory.get_container_item(construction.container)?;
                let material_index = item.kind.functions.as_material()?;
                let material = Material(material_index);
                let tile = construction.cell;

                let use_items = self.inventory.use_items_from(construction.container)?;
                let (structure, create_wall) =
                    self.building.create_wall(farmland.grid, tile, material)?;
                let create_hole = self.physics.create_hole(farmland.space, tile)?;

                if structure == Structure::Door {
                    let events = occur![
                        use_items(),
                        create_wall(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                } else {
                    let events = occur![
                        use_items(),
                        create_wall(),
                        create_hole(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                }
            }
            Marker::Reconstruction(_structure) => {
                let tile = construction.cell;
                let grid = self.building.get_grid(construction.grid)?;
                let material = grid.cells[tile[1]][tile[0]].material;
                let (structure, create_wall) =
                    self.building.create_wall(farmland.grid, tile, material)?;
                let create_hole = self.physics.create_hole(farmland.space, tile)?;

                if structure == Structure::Door {
                    let events = occur![
                        // use_items(),
                        create_wall(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                } else {
                    let events = occur![
                        // use_items(),
                        create_wall(),
                        create_hole(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                }
            }
            Marker::Deconstruction => {
                let tile = construction.cell;
                let use_items = self.inventory.use_items_from(construction.container)?;
                let destroy_wall = self.building.destroy_walls(farmland.grid, vec![tile])?;
                let destroy_hole = self.physics.destroy_hole(farmland.space, tile)?;

                let events = occur![
                    use_items(),
                    destroy_wall(),
                    destroy_hole(),
                    self.universe.vanish_construction(construction),
                ];
                Ok(events)
            }
        }
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
        stream.extend(events);
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
