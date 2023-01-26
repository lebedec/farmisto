use log::info;

use crate::building::{Grid, GridId, GridKey, GridKind};
use crate::collections::Shared;
use crate::inventory::{
    Container, ContainerId, ContainerKey, ContainerKind, Item, ItemId, ItemKey, ItemKind,
};
use crate::model::{
    Drop, Farmer, FarmerKey, FarmerKind, Farmland, FarmlandKey, FarmlandKind, Player, PlayerId,
    Tree, TreeKey, TreeKind,
};
use crate::physics::{
    Barrier, BarrierId, BarrierKey, BarrierKind, Body, BodyId, BodyKey, BodyKind, Space, SpaceId,
    SpaceKey, SpaceKind,
};
use crate::planting::{Land, LandId, LandKey, LandKind, Plant, PlantId, PlantKey, PlantKind};
use crate::Game;

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
        for kind in storage.find_all(|row| self.load_tree_kind(row).unwrap()) {
            self.universe.known.trees.insert(kind.id, Shared::new(kind));
        }
        for kind in storage.find_all(|row| self.load_farmland_kind(row).unwrap()) {
            self.universe
                .known
                .farmlands
                .insert(kind.id, Shared::new(kind));
        }
        for kind in storage.find_all(|row| self.load_farmer_kind(row).unwrap()) {
            self.universe
                .known
                .farmers
                .insert(kind.id, Shared::new(kind));
        }
        info!("End game knowledge loading");
    }

    pub fn load_game_state(&mut self) {
        info!("Begin game state loading from ...");
        let storage = self.storage.open_into();
        self.players = storage.find_all(|row| self.load_player(row).unwrap());
        // physics
        self.physics.spaces = storage.find_all(|row| self.load_space(row).unwrap());
        for body in storage.find_all(|row| self.load_body(row).unwrap()) {
            self.physics.bodies[body.space.0].push(body);
        }
        for barrier in storage.find_all(|row| self.load_barrier(row).unwrap()) {
            self.physics.barriers[barrier.space.0].push(barrier);
        }
        // planting
        self.planting.lands = storage.find_all(|row| self.load_land(row).unwrap());
        for plant in storage.find_all(|row| self.load_plant(row).unwrap()) {
            self.planting.plants[plant.land.0].push(plant);
        }
        // building
        self.building.grids = storage.find_all(|row| self.load_grid(row).unwrap());
        // inventory
        self.inventory.containers = storage.find_all(|row| self.load_container(row).unwrap());
        for item in storage.find_all(|row| self.load_item(row).unwrap()) {
            let items = self.inventory.items.entry(item.container).or_insert(vec![]);
            items.push(item);
        }
        // models
        self.universe.trees = storage.find_all(|row| self.load_tree(row).unwrap());
        self.universe.farmlands = storage.find_all(|row| self.load_farmland(row).unwrap());
        self.universe.farmers = storage.find_all(|row| self.load_farmer(row).unwrap());
        self.universe.drops = storage.find_all(|row| self.load_drop(row).unwrap());
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

    pub(crate) fn load_farmland(&mut self, row: &rusqlite::Row) -> Result<Farmland, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let space = row.get("space")?;
        let land = row.get("land")?;
        let grid = row.get("grid")?;
        let data = Farmland {
            id,
            kind: FarmlandKey(kind),
            space: SpaceId(space),
            land: LandId(land),
            grid: GridId(grid),
        };
        Ok(data)
    }

    pub(crate) fn load_farmer_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<FarmerKind, DataError> {
        let id = row.get("id")?;
        let body = row.get("body")?;
        let data = FarmerKind {
            id: FarmerKey(id),
            name: row.get("name")?,
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
            hands: ContainerId(row.get("hands")?),
            backpack: ContainerId(row.get("backpack")?),
        };
        Ok(data)
    }

    pub(crate) fn load_tree_kind(&mut self, row: &rusqlite::Row) -> Result<TreeKind, DataError> {
        let id = row.get("id")?;
        let barrier = row.get("barrier")?;
        let plant = row.get("plant")?;
        let data = TreeKind {
            id: TreeKey(id),
            name: row.get("name")?,
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

    pub(crate) fn load_drop(&mut self, row: &rusqlite::Row) -> Result<Drop, DataError> {
        let data = Drop {
            id: row.get("id")?,
            barrier: BarrierId(row.get("barrier")?),
            container: ContainerId(row.get("container")?),
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

    pub(crate) fn load_space(&mut self, row: &rusqlite::Row) -> Result<Space, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
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

    pub(crate) fn load_body(&mut self, row: &rusqlite::Row) -> Result<Body, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let space = row.get("space")?;
        let position: String = row.get("position")?;
        let direction: String = row.get("direction")?;
        let data = Body {
            id: BodyId(id),
            kind: self
                .physics
                .known_bodies
                .get(&BodyKey(kind))
                .unwrap()
                .clone(),
            position: serde_json::from_str(&position)?,
            direction: serde_json::from_str(&direction)?,
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

    pub(crate) fn load_barrier(&mut self, row: &rusqlite::Row) -> Result<Barrier, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let space = row.get("space")?;
        let position: String = row.get("position")?;
        let data = Barrier {
            id: BarrierId(id),
            kind: self
                .physics
                .known_barriers
                .get(&BarrierKey(kind))
                .unwrap()
                .clone(),
            position: serde_json::from_str(&position)?,
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
        let functions: String = row.get("functions")?;
        let data = ItemKind {
            id: ItemKey(row.get("id")?),
            name: row.get("name")?,
            functions: serde_json::from_str(&functions)?,
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

    pub(crate) fn load_plant(&mut self, row: &rusqlite::Row) -> Result<Plant, DataError> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let land = row.get("land")?;
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
