extern crate alloc;
extern crate core;

use datamap::Storage;
pub use domains::*;

use crate::api::ActionError::{ConstructionContainsUnexpectedItem, PlayerFarmerNotFound};
use crate::api::{Action, ActionError, ActionResponse, Event, FarmerBound};
use crate::building::{BuildingDomain, GridId, Marker, Material, SurveyorId};
use crate::inventory::{Function, InventoryDomain, InventoryError, Item, ItemId};
use crate::math::VectorMath;
use crate::model::Activity::Idle;
use crate::model::{Activity, Creature, CreatureKey, Crop, CropKey, Drop};
use crate::model::{Construction, Farmer, Universe};
use crate::model::{Equipment, ItemRep};
use crate::model::{EquipmentKey, PurposeDescription, UniverseDomain};
use crate::model::{Farmland, Knowledge};
use crate::model::{Player, Purpose};
use crate::model::{UniverseError, UniverseSnapshot};
use crate::physics::{BarrierId, BodyId, PhysicsDomain, SensorId};
use crate::planting::{PlantId, PlantingDomain};
use crate::raising::{AnimalId, RaisingDomain};

pub mod api;
pub mod collections;
mod data;
mod domains;
pub mod math;
pub mod model;

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
                let farmer = self
                    .universe
                    .farmers
                    .iter()
                    .find(|farmer| farmer.player == player)
                    .ok_or(PlayerFarmerNotFound(player_name.to_string()))?;
                let farmland = self.universe.farmlands[0];

                match action {
                    FarmerBound::Move { destination } => self.move_farmer(*farmer, destination)?,
                    FarmerBound::Survey {
                        surveyor,
                        tile,
                        marker,
                    } => self.survey(*farmer, surveyor, tile, marker)?,
                    FarmerBound::Construct { construction } => {
                        self.construct(*farmer, farmland, construction)?
                    }
                    FarmerBound::Deconstruct { tile } => {
                        self.deconstruct(*farmer, farmland, tile)?
                    }
                    FarmerBound::RemoveConstruction { construction } => {
                        self.remove_construction(*farmer, farmland, construction)?
                    }
                    FarmerBound::TakeItem { drop } => self.take_item(*farmer, drop)?,
                    FarmerBound::DropItem { tile } => self.drop_item(*farmer, tile)?,
                    FarmerBound::PutItem { drop } => self.put_item(*farmer, drop)?,
                    FarmerBound::TakeMaterial { construction } => {
                        self.take_material(*farmer, construction)?
                    }
                    FarmerBound::PutMaterial { construction } => {
                        self.put_material(*farmer, construction)?
                    }
                    FarmerBound::ToggleBackpack => self.toggle_backpack(*farmer)?,
                    FarmerBound::Uninstall { equipment } => {
                        self.uninstall_equipment(*farmer, farmland, equipment)?
                    }
                    FarmerBound::Install { tile } => {
                        self.install_equipment(*farmer, farmland, tile)?
                    }
                    FarmerBound::UseEquipment { equipment } => {
                        self.use_equipment(*farmer, equipment)?
                    }
                    FarmerBound::CancelActivity => self.cancel_activity(*farmer)?,
                    FarmerBound::ToggleSurveyingOption => self.toggle_surveying_option(*farmer)?,
                    FarmerBound::PlantCrop { tile } => self.plant_crop(*farmer, farmland, tile)?,
                    FarmerBound::WaterCrop { crop } => self.water_crop(*farmer, crop)?,
                    FarmerBound::HarvestCrop { crop } => self.harvest_crop(*farmer, crop)?,
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

    fn take_nap(&mut self, creature: Creature) -> Result<Vec<Event>, ActionError> {
        let events = vec![];
        Ok(events)
    }

    fn water_crop(&mut self, farmer: Farmer, crop: Crop) -> Result<Vec<Event>, ActionError> {
        let water_plant = self.planting.water_plant(crop.plant, 0.5)?;
        let events = occur![water_plant(),];
        Ok(events)
    }

    fn harvest_crop(&mut self, farmer: Farmer, crop: Crop) -> Result<Vec<Event>, ActionError> {
        let item_kind = self.known.items.find("<harvest>").unwrap();
        let (new_harvest, capacity) = match self.inventory.get_container_item(farmer.hands) {
            Ok(item) => {
                let kind = item.as_product()?;
                if crop.key != CropKey(kind) {
                    return Err(InventoryError::ItemFunctionNotFound { id: item.id }.into());
                }
                (false, item.kind.max_quantity - item.quantity)
            }
            _ => (true, item_kind.max_quantity),
        };
        let (fruits, harvest) = self.planting.harvest_plant(crop.plant, capacity)?;
        let events = if new_harvest {
            let (_, create_item) = self.inventory.create_item(
                item_kind,
                vec![Function::Product { kind: crop.key.0 }],
                farmer.hands,
                fruits,
            )?;
            let change_activity = self.universe.change_activity(farmer, Activity::Usage);
            occur![harvest(), create_item(), change_activity,]
        } else {
            let increase_item = self.inventory.increase_item(farmer.hands, fruits)?;

            occur![harvest(), increase_item(),]
        };
        Ok(events)
    }

    fn plant_crop(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let kind = item.as_seeds()?;
        let kind = self.known.crops.get(CropKey(kind)).unwrap();
        let barrier = self.known.barriers.get(kind.barrier).unwrap();
        let sensor = self.known.sensors.get(kind.sensor).unwrap();
        let plant = self.known.plants.get(kind.plant).unwrap();
        let position = position_of(tile);
        let decrease_item = self.inventory.decrease_item(farmer.hands)?;
        let (barrier, sensor, create_barrier_sensor) =
            self.physics
                .create_barrier_sensor(farmland.space, barrier, sensor, position, false)?;
        let (plant, create_plant) = self.planting.create_plant(farmland.soil, plant, 0.0)?;
        let events = occur![
            decrease_item(),
            create_barrier_sensor(),
            create_plant(),
            self.appear_crop(kind.id, barrier, sensor, plant),
        ];
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
        farmer: Farmer,
        farmland: Farmland,
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

        let tiles = constructions
            .iter()
            .map(|construction| construction.cell)
            .collect();

        let destroy_containers = self.inventory.destroy_containers(containers, false)?;
        let destroy_markers = self.building.destroy_walls(farmland.grid, tiles)?;

        let events = occur![
            destroy_containers(),
            destroy_markers(),
            constructions
                .into_iter()
                .map(|id| self.universe.vanish_construction(id))
                .flatten()
                .collect::<Vec<Universe>>(),
        ];

        Ok(events)
    }

    fn uninstall_equipment(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        equipment: Equipment,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Idle)?;
        match equipment.purpose {
            Purpose::Surveying { surveyor } => {
                // TODO: transactional
                let teardown_constructions =
                    self.teardown_constructions(farmer, farmland, surveyor)?;

                let destroy_surveyor = self.building.destroy_surveyor(surveyor)?;
                let destroy_barrier = self.physics.destroy_barrier(equipment.barrier)?;
                let functions = vec![Function::Installation {
                    kind: equipment.kind.0,
                }];
                let kind = self.known.items.find("<equipment>").unwrap();
                let (item, create_item) =
                    self.inventory
                        .create_item(kind, functions, farmer.hands, 1)?;
                let vanish_equipment = self.universe.vanish_equipment(equipment);
                let change_activity = self.universe.change_activity(farmer, Activity::Usage);

                let mut events = teardown_constructions;
                events.extend(occur![
                    destroy_surveyor(),
                    destroy_barrier(),
                    create_item(),
                    vanish_equipment,
                    change_activity,
                ]);
                Ok(events)
            }
            Purpose::Moisture { .. } => Ok(vec![]),
        }
    }

    fn use_equipment(
        &mut self,
        farmer: Farmer,
        equipment: Equipment,
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Idle)?;
        let events = match equipment.purpose {
            Purpose::Surveying { .. } => {
                let activity = Activity::Surveying {
                    equipment,
                    selection: 0,
                };
                self.universe.change_activity(farmer, activity)
            }
            Purpose::Moisture { .. } => {
                vec![]
            }
        };
        Ok(occur![events,])
    }

    fn cancel_activity(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let events = self.universe.change_activity(farmer, Idle);
        Ok(occur![events,])
    }

    fn toggle_surveying_option(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        if let Activity::Surveying {
            equipment,
            mut selection,
        } = activity
        {
            selection = (selection + 1) % 4;
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
                        kind: EquipmentKey(0),
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

    fn install_equipment(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.universe.ensure_activity(farmer, Activity::Usage)?;
        let item = self.inventory.get_container_item(farmer.hands)?;
        let key = item.as_installation()?;
        let key = EquipmentKey(key);
        let equipment_kind = self
            .known
            .equipments
            .get(key)
            .ok_or(ActionError::EquipmentKindNotFound { key })?;
        match equipment_kind.purpose {
            PurposeDescription::Surveying { surveyor } => {
                let position = position_of(tile);
                let use_item = self.inventory.use_items_from(farmer.hands)?;
                let kind = self.known.surveyors.get(surveyor).unwrap();
                let (surveyor, create_surveyor) =
                    self.building.create_surveyor(farmland.grid, kind)?;
                let kind = self.known.barriers.find("<equipment>").unwrap();
                let (barrier, create_barrier) =
                    self.physics
                        .create_barrier(farmland.space, kind, position, false)?;
                let purpose = Purpose::Surveying { surveyor };
                let appear_equipment =
                    self.universe
                        .appear_equipment(equipment_kind.id, purpose, barrier, position);
                let change_activity = self.universe.change_activity(farmer, Activity::Idle);
                let events = occur![
                    use_item(),
                    create_surveyor(),
                    create_barrier(),
                    appear_equipment,
                    change_activity,
                ];
                Ok(events)
            }
            PurposeDescription::Moisture { .. } => Ok(vec![]),
        }
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

    fn take_item(&mut self, farmer: Farmer, drop: Drop) -> Result<Vec<Event>, ActionError> {
        let container = self.inventory.get_container(drop.container)?;
        let is_last_item = container.items.len() == 1;
        let pop_item = self.inventory.pop_item(drop.container, farmer.hands)?;
        let mut events = vec![pop_item().into()];

        if is_last_item {
            let destroy_container = self
                .inventory
                .destroy_containers(vec![drop.container], false)?;
            let destroy_barrier = self.physics.destroy_barrier(drop.barrier)?;
            events.extend([
                destroy_container().into(),
                destroy_barrier().into(),
                self.universe.vanish_drop(drop).into(),
            ])
        }

        events.push(
            self.universe
                .change_activity(farmer, Activity::Usage)
                .into(),
        );

        Ok(events)
    }

    fn put_item(&mut self, farmer: Farmer, drop: Drop) -> Result<Vec<Event>, ActionError> {
        let hands = self.inventory.get_container(farmer.hands)?;
        let is_last_item = hands.items.len() <= 1;
        let transfer = self.inventory.pop_item(farmer.hands, drop.container)?;
        let activity = if is_last_item {
            self.universe.change_activity(farmer, Activity::Idle)
        } else {
            vec![]
        };
        let events = occur![transfer(), activity,];
        Ok(events)
    }

    fn drop_item(&mut self, farmer: Farmer, tile: [usize; 2]) -> Result<Vec<Event>, ActionError> {
        let hands = self.inventory.get_container(farmer.hands)?;
        let is_last_item = hands.items.len() <= 1;
        let body = self.physics.get_body(farmer.body)?;
        let space = body.space;
        let barrier_kind = self.known.barriers.find("<drop>").unwrap();
        let position = position_of(tile);
        let (barrier, create_barrier) =
            self.physics
                .create_barrier(space, barrier_kind, position, false)?;
        let container_kind = self.known.containers.find("<drop>").unwrap();
        let (container, extract_item) =
            self.inventory
                .extract_item(farmer.hands, -1, container_kind)?;
        let activity = if is_last_item {
            self.universe.change_activity(farmer, Activity::Idle)
        } else {
            vec![]
        };
        let events = occur![
            create_barrier(),
            extract_item(),
            self.universe.appear_drop(container, barrier, position),
            activity,
        ];
        Ok(events)
    }

    fn take_material(
        &mut self,
        farmer: Farmer,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let pop_item = self
            .inventory
            .pop_item(construction.container, farmer.hands)?;
        let events = vec![pop_item().into()];
        Ok(events)
    }

    fn put_material(
        &mut self,
        farmer: Farmer,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let hands = self.inventory.get_container(farmer.hands)?;
        let is_last_item = hands.items.len() == 1;
        let pop_item = self
            .inventory
            .pop_item(farmer.hands, construction.container)?;
        let mut events = vec![pop_item().into()];
        if is_last_item {
            events.push(self.universe.change_activity(farmer, Activity::Idle).into())
        }
        Ok(events)
    }

    fn survey(
        &mut self,
        _farmer: Farmer,
        surveyor: SurveyorId,
        tile: [usize; 2],
        marker: Marker,
    ) -> Result<Vec<Event>, ActionError> {
        let survey = self.building.survey(surveyor, tile, marker)?;
        let container_kind = self.known.containers.find("<construction>").unwrap();
        let grid = GridId(1);
        let (container, create_container) =
            self.inventory.create_container(container_kind.clone())?;
        let appearance = self
            .universe
            .appear_construction(container, grid, surveyor, tile);
        let events = occur![survey(), create_container(), appearance,];
        Ok(events)
    }

    fn remove_construction(
        &mut self,
        _farmer: Farmer,
        farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let tile = construction.cell;
        let destroy_container = self
            .inventory
            .destroy_containers(vec![construction.container], false)?;
        let destroy_marker = self.building.destroy_walls(farmland.grid, vec![tile])?;
        let events = occur![
            destroy_container(),
            destroy_marker(),
            self.universe.vanish_construction(construction),
        ];
        Ok(events)
    }

    fn construct(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let container = self.inventory.get_container(construction.container)?;
        let mut keywords = Vec::new();
        for item in &container.items {
            for function in &item.functions {
                if let Function::Material { keyword } = function {
                    keywords.push(*keyword);
                } else {
                    return Err(ConstructionContainsUnexpectedItem(construction));
                }
            }
        }
        // let material = self.building.index_material(farmland.grid, keywords)?;
        let material = Material(*keywords.get(0).unwrap_or(&0usize) as u8);
        let tile = construction.cell;

        let use_items = self.inventory.use_items_from(construction.container)?;
        let (marker, create_wall) = self.building.create_wall(farmland.grid, tile, material)?;
        let create_hole = self.physics.create_hole(farmland.space, tile)?;

        if marker == Marker::Door {
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

    fn deconstruct(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let destroy_wall = self.building.destroy_walls(farmland.grid, vec![tile])?;
        let destroy_hole = self.physics.destroy_hole(farmland.space, tile)?;

        let events = occur![destroy_wall(), destroy_hole(),];
        Ok(events)
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let physics_events = self.physics.update(time);

        for crop in &mut self.universe.crops {
            let sensor = self.physics.get_sensor(crop.sensor).unwrap();
            let mut impact = [0.0, 0.0];
            for signal in &sensor.signals {
                impact = impact.add(*signal);
            }
            impact = impact.normalize().neg();

            self.planting
                .integrate_impact(crop.plant, impact[0])
                .unwrap();
        }

        occur![physics_events, self.planting.update(time),]
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

    pub fn look_at_crop(&self, entity: Crop) -> Universe {
        let plant = self.planting.get_plant(entity.plant).unwrap();
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        Universe::CropAppeared {
            entity,
            impact: plant.impact,
            thirst: plant.thirst,
            hunger: plant.hunger,
            growth: plant.growth,
            health: plant.health,
            fruits: plant.fruits,
            position: barrier.position,
        }
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

    pub fn look_at_creature(&self, entity: Creature) -> Universe {
        let animal = self.raising.get_animal(entity.animal).unwrap();
        let body = self.physics.get_body(entity.body).unwrap();
        Universe::CreatureAppeared {
            entity,
            space: body.space,
            health: animal.health,
            hunger: animal.hunger,
            position: body.position,
        }
    }

    /// # Safety
    ///
    /// Relational database as source of data guarantees
    /// that domain objects exists while exist game model.
    /// So, we can unwrap references without check.
    pub fn look_around(&self, snapshot: UniverseSnapshot) -> Vec<Event> {
        let mut stream = vec![];

        for farmland in self.universe.farmlands.iter() {
            if snapshot.whole || snapshot.farmlands.contains(&farmland.id) {
                let land = self.planting.get_soil(farmland.soil).unwrap();
                let grid = self.building.get_grid(farmland.grid).unwrap();
                let space = self.physics.get_space(farmland.space).unwrap();
                stream.push(Universe::FarmlandAppeared {
                    farmland: *farmland,
                    map: land.map.clone(),
                    cells: grid.cells.clone(),
                    rooms: grid.rooms.clone(),
                    holes: space.holes.clone(),
                })
            }
        }
        // let events = snapshot
        //     .farmlands_to_delete
        //     .into_iter()
        //     .map(Universe::FarmlandVanished);
        // stream.extend(events);

        for tree in self.universe.trees.iter() {
            if snapshot.whole || snapshot.trees.contains(&tree.id) {
                let barrier = self.physics.get_barrier(tree.barrier).unwrap();
                // let plant_kind = self.planting.known_plants.get(&tree.kind.plant).unwrap();
                // stream.push(Universe::BarrierHintAppeared {
                //     id: barrier.id,
                //     kind: barrier.kind.id,
                //     position: barrier.position,
                //     bounds: barrier.kind.bounds,
                // });
                // stream.push(Universe::TreeAppeared {
                //     tree: *tree,
                //     position: barrier.position,
                //     growth: plant_kind.growth,
                // })
            }
        }
        // let events = snapshot
        //     .trees_to_delete
        //     .into_iter()
        //     .map(Universe::TreeVanished);
        // stream.extend(events);

        for farmer in self.universe.farmers.iter() {
            if snapshot.whole || snapshot.farmers.contains(&farmer.id) {
                let body = self.physics.get_body(farmer.body).unwrap();
                let player = self
                    .players
                    .iter()
                    .find(|player| player.id == farmer.player)
                    .unwrap();
                stream.push(Universe::FarmerAppeared {
                    farmer: *farmer,
                    player: player.name.clone(),
                    position: body.position,
                })
            }
        }
        // let events = snapshot
        //     .farmers_to_delete
        //     .into_iter()
        //     .map(Universe::FarmerVanished);
        // stream.extend(events);

        for drop in &self.universe.drops {
            let barrier = self.physics.get_barrier(drop.barrier).unwrap();
            stream.push(Universe::DropAppeared {
                drop: *drop,
                position: barrier.position,
            })
        }

        for construction in &self.universe.constructions {
            stream.push(Universe::ConstructionAppeared {
                id: *construction,
                cell: construction.cell,
            })
        }

        for crop in &self.universe.crops {
            stream.push(self.look_at_crop(*crop));
        }

        for creature in &self.universe.creatures {
            stream.push(self.look_at_creature(*creature));
        }

        for equipment in &self.universe.equipments {
            let barrier = self.physics.get_barrier(equipment.barrier).unwrap();
            stream.push(Universe::EquipmentAppeared {
                entity: *equipment,
                position: barrier.position,
            })
        }

        let mut items_appearance = vec![];
        for container in self.inventory.containers.values() {
            for item in &container.items {
                items_appearance.push(ItemRep {
                    id: item.id,
                    kind: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                    functions: item.functions.clone(),
                })
            }
        }
        stream.push(Universe::ItemsAppeared {
            items: items_appearance,
        });

        vec![Event::Universe(stream)]
    }

    pub fn load_game_full(&mut self) {
        self.load_game_knowledge();
        self.load_game_state().unwrap();
    }
}

#[inline]
fn position_of(tile: [usize; 2]) -> [f32; 2] {
    [tile[0] as f32 + 0.5, tile[1] as f32 + 0.5]
}
