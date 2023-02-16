extern crate alloc;
extern crate core;

use datamap::Storage;
pub use domains::*;
use std::collections::HashSet;
use std::ptr::eq;

use crate::api::ActionError::{ConstructionContainsUnexpectedItem, PlayerFarmerNotFound};
use crate::api::{Action, ActionError, Event};
use crate::building::{BuildingDomain, GridId, Marker, Material, SurveyorId};
use crate::inventory::{Function, InventoryDomain};
use crate::model::UniverseDomain;
use crate::model::UniverseSnapshot;
use crate::model::{Construction, Farmer, Universe};
use crate::model::{Drop, Theodolite};
use crate::model::{Equipment, ItemRep};
use crate::model::{Farmland, Knowledge};
use crate::model::{Player, Purpose};
use crate::physics::{Physics, PhysicsDomain};
use crate::planting::PlantingDomain;

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
            storage,
            players: vec![],
        }
    }

    pub fn perform_action(
        &mut self,
        player_name: &str,
        action: Action,
    ) -> Result<Vec<Event>, ActionError> {
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
        let events = match action {
            Action::MoveFarmer { destination } => self.move_farmer(*farmer, destination)?,
            Action::Survey {
                surveyor,
                tile,
                marker,
            } => self.survey(*farmer, surveyor, tile, marker)?,
            Action::Construct { construction } => {
                self.construct(*farmer, farmland, construction)?
            }
            Action::Deconstruct { tile } => self.deconstruct(*farmer, farmland, tile)?,
            Action::RemoveConstruction { construction } => {
                self.remove_construction(*farmer, farmland, construction)?
            }
            Action::TakeItem { drop } => self.take_item(*farmer, drop)?,
            Action::DropItem { tile } => self.drop_item(*farmer, tile)?,
            Action::PutItem { drop } => self.put_item(*farmer, drop)?,
            Action::TakeMaterial { construction } => self.take_material(*farmer, construction)?,
            Action::PutMaterial { construction } => self.put_material(*farmer, construction)?,
            Action::ToggleBackpack => self.toggle_backpack(*farmer)?,
            Action::Uninstall { equipment } => self.uninstall_equipment(*farmer, equipment)?,
        };

        Ok(events)
    }

    fn move_farmer(
        &mut self,
        farmer: Farmer,
        destination: [f32; 2],
    ) -> Result<Vec<Event>, ActionError> {
        self.physics.move_body2(farmer.body, destination);
        Ok(vec![])
    }

    fn uninstall_equipment(
        &mut self,
        farmer: Farmer,
        equipment: Equipment,
    ) -> Result<Vec<Event>, ActionError> {
        // match equipment.purpose {
        //     Purpose::Surveying { surveyor } => {
        //         let destroy_surveyor = self.building.destroy_surveyor(surveyor)?;
        //         let destroy_barrier = self.physics.destroy_barrier(equipment.barrier)?;
        //         let function = Function::Equipment {
        //             kind: equipment.kind,
        //         };
        //         let create_item = self.inventory.create_item(function, farmer.hands)?;
        //         let events = occur![
        //             destroy_surveyor(),
        //             destroy_barrier(),
        //             create_item(),
        //             self.universe.vanish_equipment(equipment),
        //         ];
        //         Ok(events)
        //     }
        //     Purpose::Moisture { .. } => Ok(vec![]),
        // }
        Ok(vec![])
    }

    fn toggle_backpack(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let backpack = self
            .inventory
            .get_container(farmer.backpack)?
            .items
            .is_empty();
        let hands = self.inventory.get_container(farmer.hands)?.items.is_empty();
        let mut events = vec![];
        if hands && !backpack {
            let transfer = self.inventory.pop_item(farmer.backpack, farmer.hands)?;
            events.push(transfer().into());
        }
        if !hands && backpack {
            let transfer = self.inventory.pop_item(farmer.hands, farmer.backpack)?;
            events.push(transfer().into());
        }
        Ok(events)
    }

    fn take_item(&mut self, farmer: Farmer, drop: Drop) -> Result<Vec<Event>, ActionError> {
        let container = self.inventory.get_container(drop.container)?;
        let is_last_item = container.items.len() == 1;
        let pop_item = self.inventory.pop_item(drop.container, farmer.hands)?;
        let mut events = vec![pop_item().into()];

        if is_last_item {
            let destroy_container = self.inventory.destroy_container(drop.container, false)?;
            let destroy_barrier = self.physics.destroy_barrier(drop.barrier)?;
            events.extend([
                destroy_container().into(),
                destroy_barrier().into(),
                self.universe.vanish_drop(drop).into(),
            ])
        }

        Ok(events)
    }

    fn put_item(&mut self, farmer: Farmer, drop: Drop) -> Result<Vec<Event>, ActionError> {
        let transfer = self.inventory.pop_item(farmer.hands, drop.container)?;
        let events = vec![transfer().into()];
        Ok(events)
    }

    fn drop_item(&mut self, farmer: Farmer, tile: [usize; 2]) -> Result<Vec<Event>, ActionError> {
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
        let events = vec![
            create_barrier().into(),
            extract_item().into(),
            self.universe
                .appear_drop(container, barrier, position)
                .into(),
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
        let pop_item = self
            .inventory
            .pop_item(farmer.hands, construction.container)?;
        let events = vec![pop_item().into()];
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
        let appearance = self.universe.appear_construction(container, grid, tile);
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
            .destroy_container(construction.container, false)?;
        let destroy_marker = self.building.destroy_wall(farmland.grid, tile)?;
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
        let mut keywords = HashSet::new();
        for item in &container.items {
            for function in &item.kind.functions {
                if let Function::Material { keyword } = function {
                    keywords.insert(keyword);
                } else {
                    return Err(ConstructionContainsUnexpectedItem(construction));
                }
            }
        }
        let material = self.building.index_material(farmland.grid, keywords)?;
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
        let destroy_wall = self.building.destroy_wall(farmland.grid, tile)?;
        let destroy_hole = self.physics.destroy_hole(farmland.space, tile)?;

        let events = occur![destroy_wall(), destroy_hole(),];
        Ok(events)
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        vec![
            self.physics.update(time).into(),
            self.planting.update(time).into(),
        ]
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
                let land = self.planting.get_land(farmland.land).unwrap();
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

        for theodolite in &self.universe.theodolites {
            stream.push(Universe::TheodoliteAppeared {
                entity: *theodolite,
                cell: theodolite.cell,
            })
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
