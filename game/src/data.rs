use std::io::Cursor;

use log::info;

use datamap::Entry;

use crate::building::{Cell, Grid, GridId, GridKey, GridKind};
use crate::collections::Shared;
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
        for entry in self.storage.fetch_all::<SpaceKind>().into_iter() {
            let space = self.load_space_kind(entry).unwrap();
            self.physics
                .known_spaces
                .insert(space.id, Shared::new(space));
        }
        for entry in self.storage.fetch_all::<BodyKind>().into_iter() {
            let body = self.load_body_kind(entry).unwrap();
            self.physics.known_bodies.insert(body.id, Shared::new(body));
        }
        for entry in self.storage.fetch_all::<BarrierKind>().into_iter() {
            let barrier = self.load_barrier_kind(entry).unwrap();
            self.physics
                .known_barriers
                .insert(barrier.id, Shared::new(barrier));
        }

        for entry in self.storage.fetch_all::<LandKind>().into_iter() {
            let land = self.load_land_kind(entry).unwrap();
            self.planting.known_lands.insert(land.id, Shared::new(land));
        }
        for entry in self.storage.fetch_all::<PlantKind>().into_iter() {
            let plant = self.load_plant_kind(entry).unwrap();
            self.planting
                .known_plants
                .insert(plant.id, Shared::new(plant));
        }

        for entry in self.storage.fetch_all::<GridKind>().into_iter() {
            let grid = self.load_grid_kind(entry).unwrap();
            self.building.known_grids.insert(grid.id, Shared::new(grid));
        }

        for entry in self.storage.fetch_all::<TreeKind>().into_iter() {
            let tree = self.load_tree_kind(entry).unwrap();
            self.universe.known.trees.insert(tree.id, Shared::new(tree));
        }
        for farmland in self
            .storage
            .open_into()
            .fetch_all_map(|row| self.load_farmland_kind(row).unwrap())
        {
            self.universe
                .known
                .farmlands
                .insert(farmland.id, Shared::new(farmland));
        }
        for entry in self.storage.fetch_all::<FarmerKind>().into_iter() {
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
        self.players = storage.fetch_all_map(|row| self.load_player(row).unwrap());
        // physics
        self.physics.spaces = self
            .storage
            .fetch_all::<Space>()
            .into_iter()
            .map(|entry| self.load_space(entry).unwrap())
            .collect();
        for entry in self.storage.fetch_all::<Body>().into_iter() {
            let body = self.load_body(entry).unwrap();
            self.physics.bodies[body.space.0].push(body);
        }
        for entry in self.storage.fetch_all::<Barrier>().into_iter() {
            let barrier = self.load_barrier(entry).unwrap();
            self.physics.barriers[barrier.space.0].push(barrier);
        }
        // planting
        self.planting.lands = self
            .storage
            .open_into()
            .fetch_all_map(|row| self.load_land(row).unwrap());
        for entry in self.storage.fetch_all::<Plant>().into_iter() {
            let plant = self.load_plant(entry).unwrap();
            self.planting.plants[plant.land.0].push(plant);
        }
        // building
        self.building.grids = self
            .storage
            .open_into()
            .fetch_all_map(|row| self.load_grid(row).unwrap());
        // models
        self.universe.trees = self
            .storage
            .open_into()
            .fetch_all_map(|row| self.load_tree(row).unwrap());
        self.universe.farmlands = self
            .storage
            .fetch_all::<Farmland>()
            .into_iter()
            .map(|entry| self.load_farmland(entry).unwrap())
            .collect();
        self.universe.farmers = self
            .storage
            .open_into()
            .fetch_all_map(|row| self.load_farmer(row).unwrap());
        info!("End game state loading")
    }

    pub fn save_game(&mut self) {}

    pub(crate) fn load_player(&mut self, row: &rusqlite::Row) -> Result<Player, rusqlite::Error> {
        let data = Player {
            id: PlayerId(row.get("id")?),
            name: row.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_farmland_kind(
        &mut self,
        row: &rusqlite::Row,
    ) -> Result<FarmlandKind, rusqlite::Error> {
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

    pub(crate) fn load_farmland(&mut self, entry: Entry) -> Result<Farmland, serde_json::Error> {
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

    pub(crate) fn load_farmer_kind(
        &mut self,
        entry: Entry,
    ) -> Result<FarmerKind, serde_json::Error> {
        let id = entry.get("id")?;
        let body = entry.get("body")?;
        let data = FarmerKind {
            id: FarmerKey(id),
            name: entry.get("name")?,
            body: BodyKey(body),
        };
        Ok(data)
    }

    pub(crate) fn load_farmer(&mut self, row: &rusqlite::Row) -> Result<Farmer, rusqlite::Error> {
        let data = Farmer {
            id: row.get("id")?,
            kind: FarmerKey(row.get("kind")?),
            player: PlayerId(row.get("player")?),
            body: BodyId(row.get("body")?),
        };
        Ok(data)
    }

    pub(crate) fn load_tree_kind(&mut self, entry: Entry) -> Result<TreeKind, serde_json::Error> {
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

    pub(crate) fn load_tree(&mut self, row: &rusqlite::Row) -> Result<Tree, rusqlite::Error> {
        let data = Tree {
            id: row.get("id")?,
            kind: TreeKey(row.get("kind")?),
            plant: PlantId(row.get("plant")?),
            barrier: BarrierId(row.get("barrier")?),
        };
        Ok(data)
    }

    // physics

    pub(crate) fn load_space_kind(&mut self, entry: Entry) -> Result<SpaceKind, serde_json::Error> {
        let id = entry.get("id")?;
        let data = SpaceKind {
            id: SpaceKey(id),
            name: entry.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_space(&mut self, entry: Entry) -> Result<Space, serde_json::Error> {
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

    pub(crate) fn load_body_kind(&mut self, entry: Entry) -> Result<BodyKind, serde_json::Error> {
        let id = entry.get("id")?;
        let data = BodyKind {
            id: BodyKey(id),
            name: entry.get("name")?,
            speed: entry.get("speed")?,
        };
        Ok(data)
    }

    pub(crate) fn load_body(&mut self, entry: Entry) -> Result<Body, serde_json::Error> {
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
        entry: Entry,
    ) -> Result<BarrierKind, serde_json::Error> {
        let id = entry.get("id")?;
        let data = BarrierKind {
            id: BarrierKey(id),
            name: entry.get("name")?,
            bounds: entry.get("bounds")?,
        };
        Ok(data)
    }

    pub(crate) fn load_barrier(&mut self, entry: Entry) -> Result<Barrier, serde_json::Error> {
        let id = entry.get("id")?;
        let kind = entry.get("kind")?;
        let space = entry.get("space")?;
        let data = Barrier {
            id: BarrierId(id),
            kind: self
                .physics
                .known_barriers
                .get(&BarrierKey(kind))
                .unwrap()
                .clone(),
            position: entry.get("position")?,
            space: SpaceId(space),
        };
        Ok(data)
    }

    // building

    pub(crate) fn load_grid_kind(&mut self, entry: Entry) -> Result<GridKind, serde_json::Error> {
        let id = entry.get("id")?;
        let data = GridKind {
            id: GridKey(id),
            name: entry.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_grid(&mut self, row: &rusqlite::Row) -> Result<Grid, rusqlite::Error> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let map: Vec<u8> = row.get("map")?;
        let mut unpacker = Unpacker::new(map);
        let [size_y, size_x]: [u32; 2] = unpacker.read();
        let mut cells = Vec::with_capacity(128);
        for y in 0..size_y {
            let mut row = Vec::with_capacity(128);
            for x in 0..size_x {
                let [wall, inner, door, window]: [u8; 4] = unpacker.read();
                row.push(Cell {
                    wall: wall == 1,
                    inner: inner == 1,
                    door: door == 1,
                    window: window == 1,
                    material: Default::default(),
                });
            }
            cells.push(row);
        }
        let rooms = Grid::calculate_shapes(&cells);
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

    // planting

    pub(crate) fn load_land_kind(&mut self, entry: Entry) -> Result<LandKind, serde_json::Error> {
        let id = entry.get("id")?;
        let data = LandKind {
            id: LandKey(id),
            name: entry.get("name")?,
        };
        Ok(data)
    }

    pub(crate) fn load_land(&mut self, row: &rusqlite::Row) -> Result<Land, rusqlite::Error> {
        let id = row.get("id")?;
        let kind = row.get("kind")?;
        let map: Vec<u8> = row.get("map")?;
        let mut unpacker = Unpacker::new(map);
        let [size_y, size_x]: [u32; 2] = unpacker.read();
        let mut map = vec![];
        for _ in 0..size_y {
            let mut row = vec![];
            for _ in 0..size_x {
                let [capacity, moisture, _, _]: [f32; 4] = unpacker.read();
                row.push([capacity, moisture]);
            }
            map.push(row);
        }
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

    pub(crate) fn load_plant_kind(&mut self, entry: Entry) -> Result<PlantKind, serde_json::Error> {
        let id = entry.get("id")?;
        let data = PlantKind {
            id: PlantKey(id),
            name: entry.get("name")?,
            growth: entry.get("growth")?,
        };
        Ok(data)
    }

    pub(crate) fn load_plant(&mut self, entry: Entry) -> Result<Plant, serde_json::Error> {
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

struct Unpacker {
    cursor: Cursor<Vec<u8>>,
}

impl Unpacker {
    pub fn new(data: Vec<u8>) -> Self {
        let cursor = Cursor::new(data);
        Unpacker { cursor }
    }

    pub fn read<T: bincode::Decode>(&mut self) -> T {
        let config = bincode::config::standard()
            .with_fixed_int_encoding()
            .skip_fixed_array_length();
        bincode::decode_from_std_read(&mut self.cursor, config).unwrap()
    }
}
