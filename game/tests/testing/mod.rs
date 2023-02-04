use datamap::Storage;
use game::building::{BuildingDomain, Grid, GridId, GridKey, GridKind, Material};
use game::collections::Shared;
use game::inventory::{Container, ContainerId, ContainerKey, ContainerKind, InventoryDomain};
use game::model::{Construction, Farmland};
use game::physics::{Space, SpaceId};
use game::planting::{Land, LandId};
use game::Game;
use std::collections::HashMap;

pub struct GameTestScenario {
    game: Game,
    farmlands: HashMap<String, Farmland>,
    constructions: HashMap<String, Construction>,
}

impl GameTestScenario {
    pub fn new() -> Self {
        let storage = Storage::open("../assets/database.sqlite").unwrap();
        let mut game = Game::new(storage);
        game.load_game_knowledge();
        GameTestScenario {
            game,
            farmlands: Default::default(),
            constructions: Default::default(),
        }
    }

    pub fn given_farmland(mut self, farmland_key: &str, farmland_name: &str) -> Self {
        let id = self.game.universe.farmlands_id + 1;
        let farmland_kind = self
            .game
            .universe
            .known
            .farmlands
            .values()
            .find(|kind| kind.name == farmland_key)
            .unwrap()
            .clone();

        let space = SpaceId(self.game.physics.spaces_sequence + 1);
        let kind = self
            .game
            .physics
            .known_spaces
            .get(&farmland_kind.space)
            .unwrap()
            .clone();
        let space_component = Space { id: space, kind };
        self.game
            .physics
            .load_spaces(vec![space_component], space.0);

        let land = LandId(self.game.planting.lands_sequence + 1);
        let kind = self
            .game
            .planting
            .known_lands
            .get(&farmland_kind.land)
            .unwrap()
            .clone();
        let land_component = Land {
            id: land,
            kind,
            map: vec![],
        };
        self.game.planting.load_lands(vec![land_component], land.0);

        let grid = GridId(self.game.building.grids_sequence + 1);
        let kind = self
            .game
            .building
            .known_grids
            .get(&farmland_kind.grid)
            .unwrap()
            .clone();
        let grid_component = Grid {
            id: grid,
            kind,
            cells: vec![],
            rooms: vec![],
        };
        self.game.building.load_grids(vec![grid_component], grid.0);

        let farmland = Farmland {
            id,
            kind: farmland_kind.id,
            space,
            land,
            grid,
        };
        self.game.universe.load_farmlands(vec![farmland], id);
        self.farmlands.insert(farmland_name.to_string(), farmland);
        self
    }

    pub fn given_construction(
        mut self,
        construction_name: &str,
        farmland_name: &str,
        cell: [usize; 2],
    ) -> Self {
        let farmland = self.farmlands.get(farmland_name).unwrap();
        let grid = farmland.grid;

        let kind = self
            .game
            .inventory
            .known_containers
            .values()
            .find(|kind| kind.name == "<construction>")
            .unwrap()
            .clone();
        let container = ContainerId(self.game.inventory.containers_sequence + 1);
        let container_component = Container {
            id: container,
            kind,
        };
        self.game
            .inventory
            .load_containers(vec![container_component], container.0);

        let id = self.game.universe.constructions_id + 1;
        let construction = Construction {
            id,
            container,
            grid,
            cell,
        };
        self.game
            .universe
            .load_constructions(vec![construction], id);
        self.constructions
            .insert(construction_name.to_string(), construction);
        self
    }
}

pub struct InventoryTestScenario {
    domain: InventoryDomain,
    containers: HashMap<String, ContainerId>,
    container_kinds: HashMap<String, ContainerKey>,
}

impl InventoryTestScenario {
    pub fn new() -> Self {
        Self {
            domain: Default::default(),
            containers: Default::default(),
            container_kinds: Default::default(),
        }
    }

    pub fn given_container_kind(mut self, container_kind: &str, capacity: usize) -> Self {
        let container_key = ContainerKey(0);
        self.domain.known_containers.insert(
            container_key,
            Shared::new(ContainerKind {
                id: container_key,
                name: container_kind.to_string(),
                capacity,
            }),
        );
        self.container_kinds
            .insert(container_kind.to_string(), container_key);
        self
    }

    pub fn given_container(mut self, kind: &str, container_name: &str) -> Self {
        let container_key = self.container_kinds.get(kind).unwrap();
        let kind = self
            .domain
            .known_containers
            .get(&container_key)
            .unwrap()
            .clone();
        let id = ContainerId(self.domain.containers_sequence + 1);
        let container = Container { id, kind };
        self.domain.load_containers(vec![container], id.0);
        self.containers.insert(container_name.to_string(), id);
        self
    }
}

pub struct BuildingTestScenario {
    domain: BuildingDomain,
    grids: HashMap<String, GridId>,
    grid_kinds: HashMap<String, GridKey>,
}

impl BuildingTestScenario {
    pub fn new() -> Self {
        Self {
            domain: BuildingDomain::default(),
            grids: Default::default(),
            grid_kinds: Default::default(),
        }
    }

    pub fn given_grid_kind(mut self, grid_kind: &str) -> Self {
        let grid_key = GridKey(0);
        self.domain.known_grids.insert(
            grid_key,
            Shared::new(GridKind {
                id: grid_key,
                name: grid_kind.to_string(),
            }),
        );
        self.grid_kinds.insert(grid_kind.to_string(), grid_key);
        self
    }

    pub fn given_grid(mut self, kind: &str, grid: &str) -> Self {
        let grid_key = self.grid_kinds.get(kind).unwrap();
        let grid_id = self
            .domain
            .create_grid(self.domain.known_grids.get(&grid_key).unwrap().clone());
        self.grids.insert(grid.to_string(), grid_id);
        self
    }

    pub fn when_player_builds_on(mut self, grid: &str, building_map: &str) -> Self {
        let grid_id = self.grids.get(grid).unwrap();
        for (y, line) in building_map.lines().skip(1).enumerate() {
            for (x, code) in line.trim().split_whitespace().enumerate() {
                match code {
                    "#" => {
                        self.domain.create_wall(*grid_id, [x, y], Material(0));
                    }
                    _ => {}
                }
            }
        }
        self
    }

    pub fn then_grid_rooms_should_be(self, _grid: &str, _expected: &str) -> Self {
        self
    }
}
