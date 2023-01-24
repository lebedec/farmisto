use std::collections::HashMap;
use std::io::Cursor;

use datamap::Entry;
use log::info;

use crate::building::{Cell, Grid, GridId, GridKey, GridKind};
use crate::collections::Shared;
use crate::inventory::{
    Container, ContainerId, ContainerKey, ContainerKind, Function, Item, ItemId, ItemKey, ItemKind,
};
use crate::model::{FarmerKey, FarmlandKey, Player, PlayerId, TreeKey};
use crate::physics::{
    Barrier, BarrierId, BarrierKey, BarrierKind, Body, BodyId, BodyKey, BodyKind, Space, SpaceId,
    SpaceKey, SpaceKind,
};
use crate::planting::{Land, LandId, LandKey, LandKind, Plant, PlantId, PlantKey, PlantKind};
use crate::{Farmer, FarmerKind, Farmland, FarmlandKind, Game, Tree, TreeKind};

impl Game {
    pub fn load_game_knowledge(&mut self) {
        info!("Begin game knowledge loading from ...");
        let storage = self.storage.open_into();
        // physics
        for kind in storage.find_all(|row| self.load_space_kind(row).unwrap()) {
            self.physics.known_spaces.insert(kind.id, Shared::new(kind));
        }
        for kind in storage.find_all(|row| self.load_body_kind(row).unwrap()) {
            self.physics.known_bodies.insert(kind.id, Shared::new(kind));
        }
        for kind in storage.find_all(|row| self.load_barrier_kind(row).unwrap()) {
            self.physics
                .known_barriers
                .insert(kind.id, Shared::new(kind));
        }
        // planting
        for kind in storage.find_all(|row| self.load_land_kind(row).unwrap()) {
            self.planting.known_lands.insert(kind.id, Shared::new(kind));
        }
        for kind in storage.find_all(|row| self.load_plant_kind(row).unwrap()) {
            self.planting
                .known_plants
                .insert(kind.id, Shared::new(kind));
        }
        // building
        for kind in storage.find_all(|row| self.load_grid_kind(row).unwrap()) {
            self.building.known_grids.insert(kind.id, Shared::new(kind));
        }
        // inventory
        for kind in storage.find_all(|row| self.load_container_kind(row).unwrap()) {
            self.inventory
                .known_containers
                .insert(kind.id, Shared::new(kind));
        }
        for kind in storage.find_all(|row| self.load_item_kind(row).unwrap()) {
            self.inventory
                .known_items
                .insert(kind.id, Shared::new(kind));
        }
        // universe
        for entry in storage.fetch_all::<TreeKind>().into_iter() {
            let tree = self.load_tree_kind(entry).unwrap();
            self.universe.known.trees.insert(tree.id, Shared::new(tree));
        }
        for farmland in storage.find_all(|row| self.load_farmland_kind(row).unwrap()) {
            self.universe
                .known
                .farmlands
                .insert(farmland.id, Shared::new(farmland));
        }
        for entry in storage.fetch_all::<FarmerKind>().into_iter() {
            let farmer = self.load_farmer_kind(entry).unwrap();
            self.universe
                .known
                .farmers
                .insert(farmer.id, Shared::new(farmer));
        }
        info!("End game knowledge loading");
    }

    pub fn load_game_state(&mut self) {
        info!("Begin game state loading from ...");
        let storage = self.storage.open_into();
        self.players = storage.find_all(|row| self.load_player(row).unwrap());
        // physics
        self.physics.spaces = storage
            .fetch_all::<Space>()
            .into_iter()
            .map(|entry| self.load_space(entry).unwrap())
            .collect();
        for entry in storage.fetch_all::<Body>().into_iter() {
            let body = self.load_body(entry).unwrap();
            self.physics.bodies[body.space.0].push(body);
        }
        for entry in storage.fetch_all::<Barrier>().into_iter() {
            let barrier = self.load_barrier(entry).unwrap();
            self.physics.barriers[barrier.space.0].push(barrier);
        }
        // planting
        self.planting.lands = storage.find_all(|row| self.load_land(row).unwrap());
        for entry in storage.fetch_all::<Plant>().into_iter() {
            let plant = self.load_plant(entry).unwrap();
            self.planting.plants[plant.land.0].push(plant);
        }
        // building
        self.building.grids = storage.find_all(|row| self.load_grid(row).unwrap());
        // inventory
        self.inventory.containers = storage.find_all(|row| self.load_container(row).unwrap());
        for item in storage.find_all(|row| self.load_item(row).unwrap()) {
            self.inventory
                .items
                .entry(item.container)
                .and_modify(|container| container.push(item));
        }
        // models
        self.universe.trees = storage.find_all(|row| self.load_tree(row).unwrap());
        self.universe.farmlands = storage
            .fetch_all::<Farmland>()
            .into_iter()
            .map(|entry| self.load_farmland(entry).unwrap())
            .collect();
        self.universe.farmers = storage.find_all(|row| self.load_farmer(row).unwrap());
        info!("End game state loading")
    }

    pub fn save_game(&mut self) {}

    pub(crate) fn load_player(&mut self, row: &rusqlite::Row) -> Result<Player, DataError> {
        let data = Player {
            id: PlayerId(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_farmland_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<FarmlandKind, DataError> {
        let id = row.get("id")?;
        let space = row.get("space")?;
        let land = row.get("land")?;
        let grid = row.get("grid")?;
        let data = FarmlandKind {
            id: FarmlandKey(id),
            name: row.get("name")?,
            space: SpaceKey(space),
            land: LandKey(land),
            grid: GridKey(grid),
        };
        Ok(data)
    }

    pub(crate) fn load_farmland(&mut self, entry: Entry) -> Result<Farmland, DataError> {
        let id = entry.get("id")?;
        let kind = entry.get("kind")?;
        let space = entry.get("space")?;
        let land = entry.get("land")?;
        let grid = entry.get("grid")?;
        let data = Farmland {
            id,
            kind: FarmlandKey(kind),
            space: SpaceId(space),
            land: LandId(land),
            grid: GridId(grid),
        };
        Ok(data)
    }

    pub(crate) fn load_farmer_kind(&mut self, entry: Entry) -> Result<FarmerKind, DataError> {
        let id = entry.get("id")?;
        let body = entry.get("body")?;
        let data = FarmerKind {
            id: FarmerKey(id),
            name: entry.get("name")?,
            body: BodyKey(body),
        };
        Ok(data)
    }

    pub(crate) fn load_farmer(&mut self, row: &rusqlite::Row) -> Result<Farmer, DataError> {
        let data = Farmer {
            id: row.get("id")?,
            kind: FarmerKey(row.get("kind")?),
            player: PlayerId(row.get("player")?),
            body: BodyId(row.get("body")?),
        };
        Ok(data)
    }

    pub(crate) fn load_tree_kind(&mut self, entry: Entry) -> Result<TreeKind, DataError> {
        let id = entry.get("id")?;
        let barrier = entry.get("barrier")?;
        let plant = entry.get("plant")?;
        let data = TreeKind {
            id: TreeKey(id),
            name: entry.get("name")?,
            barrier: BarrierKey(barrier),
            plant: PlantKey(plant),
        };
        Ok(data)
    }

    pub(crate) fn load_tree(&mut self, row: &rusqlite::Row) -> Result<Tree, DataError> {
        let data = Tree {
            id: row.get("id")?,
            kind: TreeKey(row.get("kind")?),
            plant: PlantId(row.get("plant")?),
            barrier: BarrierId(row.get("barrier")?),
        };
        Ok(data)
    }

    // physics

    pub(crate) fn load_space_kind(&mut self, row: &rusqlite::Row) -> Result<SpaceKind, DataError> {
        let data = SpaceKind {
            id: SpaceKey(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_space(&mut self, entry: Entry) -> Result<Space, DataError> {
        let id = entry.get("id")?;
        let kind = entry.get("kind")?;
        let data = Space {
            id: SpaceId(id),
            kind: self
                .physics
                .known_spaces
                .get(&SpaceKey(kind))
                .unwrap()
                .clone(),
        };
        Ok(data)
    }

    pub(crate) fn load_body_kind(&mut self, row: &rusqlite::Row) -> Result<BodyKind, DataError> {
        let id = row.get("id")?;
        let data = BodyKind {
            id: BodyKey(id),
            name: row.get("name")?,
            speed: row.get("speed")?,
        };
        Ok(data)
    }

    pub(crate) fn load_body(&mut self, entry: Entry) -> Result<Body, DataError> {
        let id = entry.get("id")?;
        let kind = entry.get("kind")?;
        let space = entry.get("space")?;
        let data = Body {
            id: BodyId(id),
            kind: self
                .physics
                .known_bodies
                .get(&BodyKey(kind))
                .unwrap()
                .clone(),
            position: entry.get("position")?,
            direction: entry.get("direction")?,
            space: SpaceId(space),
        };
        Ok(data)
    }

    pub(crate) fn load_barrier_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<BarrierKind, DataError> {
        let id = row.get("id")?;
        let bounds: String = row.get("bounds")?;
        let data = BarrierKind {
            id: BarrierKey(id),
            name: row.get("name")?,
            bounds: serde_json::from_str(&bounds)?,
        };
        Ok(data)
    }

    pub(crate) fn load_barrier(&mut self, row: Entry) -> Result<Barrier, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let space = row.get("space")?;
        let data = Barrier {
            id: BarrierId(id),
            kind: self
                .physics
                .known_barriers
                .get(&BarrierKey(kind))
                .unwrap()
                .clone(),
            position: row.get("position")?,
            space: SpaceId(space),
        };
        Ok(data)
    }

    // building

    pub(crate) fn load_grid_kind(&mut self, row: &rusqlite::Row) -> Result<GridKind, DataError> {
        let id = row.get("id")?;
        let data = GridKind {
            id: GridKey(id),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_grid(&mut self, row: &rusqlite::Row) -> Result<Grid, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let map: Vec<u8> = row.get("map")?;
        let config = bincode::config::standard();
        let (cells, _) = bincode::decode_from_slice(&map, config).unwrap();
        let rooms = Grid::calculate_rooms(&cells);
        let data = Grid {
            id: GridId(id),
            kind: self
                .building
                .known_grids
                .get(&GridKey(kind))
                .unwrap()
                .clone(),
            cells,
            rooms,
        };
        Ok(data)
    }

    // inventory

    pub(crate) fn load_container_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<ContainerKind, DataError> {
        let data = ContainerKind {
            id: ContainerKey(row.get("id")?),
            name: row.get("name")?,
            capacity: row.get("capacity")?,
        };
        Ok(data)
    }

    pub(crate) fn load_container(&mut self, row: &rusqlite::Row) -> Result<Container, DataError> {
        let kind = self
            .inventory
            .known_containers
            .get(&ContainerKey(row.get("kind")?))
            .unwrap()
            .clone();

        let data = Container {
            id: ContainerId(row.get("id")?),
            kind,
        };
        Ok(data)
    }

    pub(crate) fn load_item_kind(&mut self, row: &rusqlite::Row) -> Result<ItemKind, DataError> {
        let functions: Vec<u8> = row.get("tiles")?;
        let data = ItemKind {
            id: ItemKey(row.get("id")?),
            name: row.get("name")?,
            functions: serde_json::from_slice(&functions)?,
        };
        Ok(data)
    }

    pub(crate) fn load_item(&mut self, row: &rusqlite::Row) -> Result<Item, DataError> {
        let kind = self
            .inventory
            .known_items
            .get(&ItemKey(row.get("kind")?))
            .unwrap()
            .clone();

        let data = Item {
            id: ItemId(row.get("id")?),
            kind,
            container: ContainerId(row.get("container")?),
        };
        Ok(data)
    }

    // planting

    pub(crate) fn load_land_kind(&mut self, row: &rusqlite::Row) -> Result<LandKind, DataError> {
        let id = row.get("id")?;
        let data = LandKind {
            id: LandKey(id),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_land(&mut self, row: &rusqlite::Row) -> Result<Land, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let map: Vec<u8> = row.get("map")?;
        let config = bincode::config::standard();
        let (map, _) = bincode::decode_from_slice(&map, config).unwrap();
        let data = Land {
            id: LandId(id),
            kind: self
                .planting
                .known_lands
                .get(&LandKey(kind))
                .unwrap()
                .clone(),
            map,
        };
        Ok(data)
    }

    pub(crate) fn load_plant_kind(&mut self, row: &rusqlite::Row) -> Result<PlantKind, DataError> {
        let id = row.get("id")?;
        let data = PlantKind {
            id: PlantKey(id),
            name: row.get("name")?,
            growth: row.get("growth")?,
        };
        Ok(data)
    }

    pub(crate) fn load_plant(&mut self, entry: Entry) -> Result<Plant, DataError> {
        let id = entry.get("id")?;
        let kind = entry.get("kind")?;
        let land = entry.get("land")?;
        let data = Plant {
            id: PlantId(id),
            kind: self
                .planting
                .known_plants
                .get(&PlantKey(kind))
                .unwrap()
                .clone(),
            land: LandId(land),
        };
        Ok(data)
    }
}

#[derive(Debug)]
pub enum DataError {
    Json(serde_json::Error),
    Sql(rusqlite::Error),
    Consistency,
}

impl From<serde_json::Error> for DataError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<rusqlite::Error> for DataError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}
