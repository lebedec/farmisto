use datamap::Storage;
pub use domains::*;

use crate::api::ActionError::{
    ConstructionContainsUnexpectedItem, FarmerBodyNotFound, PlayerFarmerNotFound,
};
use crate::api::{Action, ActionError, Event};
use crate::building::BuildingDomain;
use crate::inventory::{ContainerKey, Function, InventoryDomain};
use crate::model::FarmlandKind;
use crate::model::Tree;
use crate::model::TreeKind;
use crate::model::UniverseDomain;
use crate::model::UniverseSnapshot;
use crate::model::{Construction, Farmer, Universe};
use crate::model::{FarmerKey, Farmland};
use crate::model::{FarmerKind, Player};
use crate::model::{Theodolite, Tile};
use crate::physics::PhysicsDomain;
use crate::planting::PlantingDomain;

pub mod api;
pub mod collections;
mod data;
mod domains;
pub mod math;
pub mod model;

pub struct Game {
    pub universe: UniverseDomain,
    physics: PhysicsDomain,
    planting: PlantingDomain,
    building: BuildingDomain,
    inventory: InventoryDomain,
    storage: Storage,
    players: Vec<Player>,
}

impl Game {
    pub fn new(storage: Storage) -> Self {
        Self {
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
        player_name: &String,
        action: Action,
    ) -> Result<Vec<Event>, ActionError> {
        let mut events = vec![];
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
            Action::DoSomething => {}
            Action::DoAnything { .. } => {}
            Action::MoveFarmer { destination } => {
                // match self
                //     .universe
                //     .farmers
                //     .iter()
                //     .find(|farmer| &farmer.player == player)
                // {
                //     Some(farmer) => {
                //         self.physics.move_body2(farmer.id.into(), destination);
                //     }
                //     None => {
                //         // error framer not found, action_id error
                //     }
                // }
            }
            Action::BuildWall { cell } => {
                unimplemented!()
            }
            Action::Construct { construction } => {
                events.extend(self.construct(*farmer, farmland, construction)?)
            }
        }
        Ok(events)
    }

    fn survey(
        &mut self,
        farmer: Farmer,
        theodolite: Theodolite,
        target: Tile,
    ) -> Result<Vec<Event>, ActionError> {
        let surveying = self
            .building
            .survey(theodolite.surveyor, [target.x, target.y])?;
        let container_kind = self
            .inventory
            .known_containers
            .get(&ContainerKey(0))
            .unwrap();
        let container_creation = self.inventory.create_container(container_kind.clone())?;
        let construction_creation = self
            .universe
            .aggregate_to_construction(container_creation.container.id, surveying.cell)?;
        let events = vec![
            Event::Building(surveying.complete()),
            Event::Inventory(container_creation.complete()),
            Event::Universe(construction_creation.complete()),
        ];
        Ok(events)
    }

    fn construct(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let body = self
            .physics
            .get_body(farmer.body)
            .ok_or(FarmerBodyNotFound(farmer))?;

        let usage = self.inventory.use_items_from(construction.container)?;

        let mut keywords = vec![];
        for item in usage.items() {
            for function in &item.kind.functions {
                if let Function::Material { keyword } = function {
                    keywords.push(keyword);
                } else {
                    return Err(ConstructionContainsUnexpectedItem(construction));
                }
            }
        }

        Ok(vec![])
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        vec![
            Event::Physics(self.physics.update(time)),
            Event::Planting(self.planting.update(time)),
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
                let grid = self.building.get_grid(farmland.grid);
                stream.push(Universe::FarmlandAppeared {
                    farmland: *farmland,
                    map: land.map.clone(),
                    cells: grid.cells.clone(),
                    rooms: grid.rooms.clone(),
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

        vec![Event::Universe(stream)]
    }

    pub fn load_game_full(&mut self) {
        self.load_game_knowledge();
        self.load_game_state();
    }
}
