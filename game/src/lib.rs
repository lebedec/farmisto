use datamap::Storage;
pub use domains::*;

use crate::api::ActionError::{
    ConstructionContainsUnexpectedItem, FarmerBodyNotFound, PlayerFarmerNotFound,
};
use crate::api::{Action, ActionError, Event};
use crate::building::{BuildingDomain, GridId, SurveyorId};
use crate::inventory::{Function, Inventory, InventoryDomain};
use crate::model::Farmland;
use crate::model::ItemView;
use crate::model::Player;
use crate::model::UniverseDomain;
use crate::model::UniverseSnapshot;
use crate::model::{Construction, Farmer, Universe};
use crate::model::{Drop, Theodolite, Tile};
use crate::physics::{PhysicsDomain, SpaceId};
use crate::planting::PlantingDomain;

pub mod api;
pub mod collections;
mod data;
mod domains;
pub mod math;
pub mod model;

pub struct Game {
    pub universe: UniverseDomain,
    pub physics: PhysicsDomain,
    pub planting: PlantingDomain,
    pub building: BuildingDomain,
    pub inventory: InventoryDomain,
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
            Action::MoveFarmer { destination } => {
                events.extend(self.move_farmer(*farmer, destination)?)
            }
            Action::BuildWall { cell: _ } => {
                unimplemented!()
            }
            Action::Survey { theodolite, tile } => {
                events.extend(self.survey(*farmer, theodolite, tile)?)
            }
            Action::Construct { construction } => {
                events.extend(self.construct(*farmer, farmland, construction)?)
            }
            Action::RemoveConstruction { construction } => {
                events.extend(self.remove_construction(*farmer, farmland, construction)?)
            }
            Action::TakeItem { drop } => events.extend(self.take_item(*farmer, drop)?),
            Action::DropItem { tile } => events.extend(self.drop_item(*farmer, tile)?),
            Action::PutItem { drop } => events.extend(self.put_item(*farmer, drop)?),
            Action::TakeMaterial { construction } => {
                events.extend(self.take_material(*farmer, construction)?)
            }
            Action::PutMaterial { construction } => {
                events.extend(self.put_material(*farmer, construction)?)
            }
            Action::ToggleBackpack => events.extend(self.toggle_backpack(*farmer)?),
        }
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

    fn toggle_backpack(&mut self, farmer: Farmer) -> Result<Vec<Event>, ActionError> {
        let backpack = self.inventory.get_items(farmer.backpack);
        let hands = self.inventory.get_items(farmer.hands);
        let events = if let Ok(items) = backpack {
            let transfer =
                self.inventory
                    .transfer_item(farmer.backpack, items[0].id, farmer.hands)?;
            vec![Event::Inventory(transfer.complete())]
        } else if let Ok(items) = hands {
            let transfer =
                self.inventory
                    .transfer_item(farmer.hands, items[0].id, farmer.backpack)?;
            vec![Event::Inventory(transfer.complete())]
        } else {
            vec![]
        };
        Ok(events)
    }

    fn take_item(&mut self, farmer: Farmer, drop: Drop) -> Result<Vec<Event>, ActionError> {
        let items = self.inventory.get_items(drop.container)?;
        let need_destroy = items.len() == 1;
        let transfer = self
            .inventory
            .transfer_item(drop.container, items[0].id, farmer.hands)?;
        let mut events = vec![Event::Inventory(transfer.complete())];
        if need_destroy {
            events.push(Event::Universe(self.universe.vanish_drop(drop)))
        }

        Ok(events)
    }

    fn put_item(&mut self, farmer: Farmer, drop: Drop) -> Result<Vec<Event>, ActionError> {
        let items = self.inventory.get_items(farmer.hands)?;
        let transfer = self
            .inventory
            .transfer_item(farmer.hands, items[0].id, drop.container)?;
        let events = vec![Event::Inventory(transfer.complete())];
        Ok(events)
    }

    fn take_material(
        &mut self,
        farmer: Farmer,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let items = self.inventory.get_items(construction.container)?;
        let transfer =
            self.inventory
                .transfer_item(construction.container, items[0].id, farmer.hands)?;
        let events = vec![Event::Inventory(transfer.complete())];
        Ok(events)
    }

    fn put_material(
        &mut self,
        farmer: Farmer,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let items = self.inventory.get_items(farmer.hands)?;
        if items.len() > 0 {
            let transfer =
                self.inventory
                    .transfer_item(farmer.hands, items[0].id, construction.container)?;
            let events = vec![Event::Inventory(transfer.complete())];
            Ok(events)
        } else {
            Ok(vec![])
        }
    }

    fn drop_item(&mut self, farmer: Farmer, tile: [usize; 2]) -> Result<Vec<Event>, ActionError> {
        let space = SpaceId(0);
        let items = self.inventory.get_items(farmer.hands)?;
        let item = items[0].id;
        let (_, barrier_kind) = self
            .physics
            .known_barriers
            .iter()
            .find(|(_, kind)| kind.name == "<drop>")
            .unwrap();
        let position = [
            (tile[0] as f32) * 128.0 + 64.0,
            (tile[1] as f32) * 128.0 + 64.0,
        ];
        let barrier_creation =
            self.physics
                .create_barrier(space, barrier_kind.clone(), position)?;
        let (_, container_kind) = self
            .inventory
            .known_containers
            .iter()
            .find(|(_, kind)| kind.name == "<drop>")
            .unwrap();
        let container_creation =
            self.inventory
                .hold_item(farmer.hands, item, container_kind.clone())?;
        let appearance = self.universe.appear_drop(
            container_creation.container.id,
            barrier_creation.barrier.id,
            position,
        );
        let events = vec![
            Event::Physics(barrier_creation.complete()),
            Event::Inventory(container_creation.complete()),
            Event::Universe(appearance),
        ];
        Ok(events)
    }

    fn survey(
        &mut self,
        _farmer: Farmer,
        _theodolite: Theodolite,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let surveying = self.building.survey(SurveyorId(0), tile)?;
        let (_, container_kind) = self
            .inventory
            .known_containers
            .iter()
            .find(|(_, kind)| kind.name == "<construction>")
            .unwrap();
        let grid = GridId(1);
        let container_creation = self.inventory.create_container(container_kind.clone())?;
        let appearance = self.universe.appear_construction(
            container_creation.container.id,
            grid,
            [surveying.cell.0, surveying.cell.1],
        );
        let events = vec![
            Event::Building(surveying.complete()),
            Event::Inventory(container_creation.complete()),
            Event::Universe(appearance),
        ];
        Ok(events)
    }

    fn remove_construction(
        &mut self,
        _farmer: Farmer,
        _farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        // let items = self.inventory.get_items(construction.container).unwrap();
        let tile = construction.cell;
        let (_, _barrier_kind) = self
            .physics
            .known_barriers
            .iter()
            .find(|(_, kind)| kind.name == "<drop>")
            .unwrap();
        let _position = [
            (tile[0] as f32) * 128.0 + 64.0,
            (tile[1] as f32) * 128.0 + 64.0,
        ];
        let events = vec![Event::Universe(
            self.universe.vanish_construction(construction),
        )];
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

        let mut items_appearance = vec![];
        for items in self.inventory.get_all_items() {
            for item in items {
                items_appearance.push(ItemView {
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
