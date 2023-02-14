use datamap::Storage;
pub use domains::*;

use crate::api::ActionError::{
    ConstructionContainsUnexpectedItem, FarmerBodyNotFound, PlayerFarmerNotFound,
};
use crate::api::{Action, ActionError, Event};
use crate::building::{BuildingDomain, GridId, SurveyorId};
use crate::inventory::{Function, Inventory, InventoryDomain};
use crate::model::ItemView;
use crate::model::Player;
use crate::model::UniverseDomain;
use crate::model::UniverseSnapshot;
use crate::model::{Construction, Farmer, Universe};
use crate::model::{Drop, Theodolite, Tile};
use crate::model::{Farmland, Knowledge};
use crate::physics::{PhysicsDomain, SpaceId};
use crate::planting::PlantingDomain;

pub mod api;
pub mod collections;
mod data;
mod domains;
pub mod math;
pub mod model;

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
            events.extend([
                destroy_container().into(),
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
        _theodolite: Theodolite,
        tile: [usize; 2],
    ) -> Result<Vec<Event>, ActionError> {
        let surveying = self.building.survey(SurveyorId(0), tile)?;
        let container_kind = self.known.containers.find("<construction>").unwrap();
        let grid = GridId(1);
        let (container, create_container) =
            self.inventory.create_container(container_kind.clone())?;
        let appearance = self.universe.appear_construction(
            container,
            grid,
            [surveying.cell.0, surveying.cell.1],
        );
        let events = vec![
            surveying.complete().into(),
            create_container().into(),
            appearance.into(),
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
        let barrier_kind = self.known.barriers.find("<drop>").unwrap();
        let position = position_of(tile);
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
        // let body = self
        //     .physics
        //     .get_body(farmer.body)
        //     .ok_or(FarmerBodyNotFound(farmer))?;

        let container = self.inventory.get_container(construction.container)?;
        let mut keywords = vec![];
        for item in &container.items {
            for function in &item.kind.functions {
                if let Function::Material { keyword } = function {
                    keywords.push(keyword);
                } else {
                    return Err(ConstructionContainsUnexpectedItem(construction));
                }
            }
        }

        let use_items = self.inventory.use_items_from(construction.container)?;

        Ok(vec![use_items().into()])
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
                let grid = self.building.get_grid(farmland.grid);
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

        let mut items_appearance = vec![];
        for container in self.inventory.containers.values() {
            for item in &container.items {
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

#[inline]
fn position_of(tile: [usize; 2]) -> [f32; 2] {
    [tile[0] as f32 + 0.5, tile[1] as f32 + 0.5]
}
