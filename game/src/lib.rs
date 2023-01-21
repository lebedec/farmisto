use datamap::Storage;
pub use domains::*;

use crate::api::ActionError::{
    ConstructionContainerNotFound, ConstructionContainerNotInitialized,
    ConstructionContainsUnexpectedItem, FarmerBodyNotFound, PlayerFarmerNotFound,
};
use crate::api::{occur, Action, ActionError, Event};
use crate::building::{encode_platform_map, Building, BuildingDomain, GridId};
use crate::inventory::{ContainerKey, Function, InventoryDomain};
use crate::model::{Construction, Farmer};
use crate::model::{FarmerId, Theodolite, Tile};

use crate::model::FarmerKind;
use crate::model::Farmland;
use crate::model::FarmlandId;

use crate::model::FarmlandKind;
use crate::model::Tree;
use crate::model::TreeId;

use crate::model::TreeKind;
use crate::model::UniverseDomain;
use crate::model::UniverseSnapshot;
use crate::physics::{Physics, PhysicsDomain};
use crate::planting::{Planting, PlantingDomain};

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
        }
    }

    pub fn perform_action(
        &mut self,
        player: &String,
        action: Action,
    ) -> Result<Vec<Event>, ActionError> {
        let mut events = vec![];
        let farmer = self
            .universe
            .farmers
            .iter()
            .find(|farmer| &farmer.player == player)
            .ok_or(PlayerFarmerNotFound(player.to_string()))?;
        let farmer = farmer.id;
        let farmland = self.universe.farmlands[0].id;
        match action {
            Action::DoSomething => {}
            Action::DoAnything { .. } => {}
            Action::MoveFarmer { destination } => {
                match self
                    .universe
                    .farmers
                    .iter()
                    .find(|farmer| &farmer.player == player)
                {
                    Some(farmer) => {
                        self.physics.move_body2(farmer.id.into(), destination);
                    }
                    None => {
                        // error framer not found, action_id error
                    }
                }
            }
            Action::BuildWall { cell } => {
                unimplemented!()
            }
            Action::Construct { construction } => {
                events.extend(self.construct(farmer, farmland, construction)?)
            }
        }
        Ok(events)
    }

    fn survey(
        &mut self,
        farmer: FarmerId,
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

        let mut events: Vec<Event> = vec![];
        events.extend(occur(surveying.complete()));
        events.extend(occur(container_creation.complete()));
        events.extend(construction_creation.complete());
        Ok(events)
    }

    fn construct(
        &mut self,
        farmer: FarmerId,
        farmland: FarmlandId,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let body = self
            .physics
            .get_body(farmer.into())
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
        let mut events = vec![];
        for event in self.physics.update(time) {
            match event {
                Physics::BodyPositionChanged { id, position, .. } => {
                    if let Some(farmer) = self
                        .universe
                        .farmers
                        .iter()
                        .find(|farmer| id == farmer.id.into())
                    {
                        events.push(Event::FarmerMoved {
                            id: farmer.id,
                            position,
                        })
                    }
                }
            }
        }
        for event in self.planting.update(time) {
            match event {
                Planting::LandChanged { id, map } => {
                    events.push(Event::FarmlandUpdated { id: id.into(), map })
                }
            }
        }
        events
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
                let land = self.planting.get_land(farmland.id.into()).unwrap();
                let platform = self.building.get_platform(farmland.id.into()).unwrap();
                stream.push(Event::FarmlandAppeared {
                    id: farmland.id,
                    kind: farmland.kind.id,
                    map: land.map.clone(),
                    platform: encode_platform_map(platform.cells),
                    platform_shapes: platform.rooms.clone(),
                })
            }
        }
        let events = snapshot
            .farmlands_to_delete
            .into_iter()
            .map(Event::FarmlandVanished);
        stream.extend(events);

        for tree in self.universe.trees.iter() {
            if snapshot.whole || snapshot.trees.contains(&tree.id) {
                let barrier = self.physics.get_barrier(tree.id.into()).unwrap();
                let plant_kind = self.planting.known_plants.get(&tree.kind.plant).unwrap();
                stream.push(Event::BarrierHintAppeared {
                    id: barrier.id.0,
                    kind: barrier.kind.id.0,
                    position: barrier.position,
                    bounds: barrier.kind.bounds,
                });
                stream.push(Event::TreeAppeared {
                    id: tree.id,
                    kind: tree.kind.id,
                    position: barrier.position,
                    growth: plant_kind.growth,
                })
            }
        }
        let events = snapshot
            .trees_to_delete
            .into_iter()
            .map(Event::TreeVanished);
        stream.extend(events);

        for farmer in self.universe.farmers.iter() {
            if snapshot.whole || snapshot.farmers.contains(&farmer.id) {
                let body = self.physics.get_body(farmer.id.into()).unwrap();
                stream.push(Event::FarmerAppeared {
                    id: farmer.id,
                    kind: farmer.kind.id,
                    player: farmer.player.clone(),
                    position: body.position,
                })
            }
        }
        let events = snapshot
            .farmers_to_delete
            .into_iter()
            .map(Event::FarmerVanished);
        stream.extend(events);

        stream
    }

    pub fn load_game_full(&mut self) {
        self.load_game_knowledge();
        self.load_game_state();
    }
}
