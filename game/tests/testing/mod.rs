use datamap::Storage;
use game::api::{Action, ActionError, Event};
use game::building::{BuildingDomain, Grid, GridId, GridKey, GridKind, Material};
use game::collections::Shared;
use game::inventory::{
    Container, ContainerId, ContainerKey, ContainerKind, InventoryDomain, Item, ItemId, ItemKey,
};
use game::model::{Construction, Drop, Farmer, Farmland, Player, PlayerId};
use game::physics::{Barrier, BarrierId, Body, BodyId, Space, SpaceId};
use game::planting::{Land, LandId};
use game::Game;
use std::collections::HashMap;

pub fn at<T>(x: T, y: T) -> [T; 2] {
    [x, y]
}

pub struct GameTestScenario {
    game: Game,
    farmlands: HashMap<String, Farmland>,
    farmers: HashMap<String, Farmer>,
    drops: HashMap<String, Drop>,
    constructions: HashMap<String, Construction>,
    containers: HashMap<String, ContainerId>,
    items: HashMap<String, ItemId>,
    current_farmland: Option<Farmland>,
    current_action_result: Result<Vec<Event>, ActionError>,
}

impl GameTestScenario {
    pub fn new() -> Self {
        let storage = Storage::open("../assets/database.sqlite").unwrap();
        let mut game = Game::new(storage);
        game.load_game_knowledge();
        GameTestScenario {
            game,
            farmlands: Default::default(),
            farmers: Default::default(),
            drops: Default::default(),
            constructions: Default::default(),
            containers: Default::default(),
            items: Default::default(),
            current_farmland: None,
            current_action_result: Err(ActionError::Test),
        }
    }

    pub fn drop(&self, name: &str) -> Drop {
        self.drops.get(name).unwrap().clone()
    }

    pub fn item_key(&self, name: &str) -> ItemKey {
        self.game.known.items.find(name).unwrap().id
    }

    pub fn item(&self, name: &str) -> ItemId {
        self.items.get(name).unwrap().clone()
    }

    pub fn container(&self, name: &str) -> ContainerId {
        self.containers.get(name).unwrap().clone()
    }

    pub fn given_farmer(mut self, farmer_key: &str, farmer_name: &str, cell: [usize; 2]) -> Self {
        let farmland = self.current_farmland.unwrap();
        let player = PlayerId(self.game.players.len());
        self.game.players.push(Player {
            id: player,
            name: farmer_name.to_string(),
        });

        let farmer_kind = self.game.known.farmers.find(farmer_key).unwrap();

        let kind = self.game.known.containers.find("<hands>").unwrap();
        let hands = ContainerId(self.game.inventory.containers_sequence + 1);
        let container_component = Container { id: hands, kind };
        self.game
            .inventory
            .load_containers(vec![container_component], hands.0);

        let kind = self.game.known.containers.find("<backpack>").unwrap();
        let backpack = ContainerId(self.game.inventory.containers_sequence + 1);
        let container_component = Container { id: backpack, kind };
        self.game
            .inventory
            .load_containers(vec![container_component], hands.0);

        let kind = self.game.known.bodies.get(farmer_kind.body).unwrap();
        let body = BodyId(self.game.physics.bodies_sequence + 1);
        let body_component = Body {
            id: body,
            kind,
            position: [0.0, 0.0],
            direction: [0.0, 0.0],
            space: farmland.space,
        };
        self.game.physics.load_bodies(vec![body_component], body.0);

        let id = self.game.universe.farmers_id + 1;
        let farmer = Farmer {
            id,
            kind: farmer_kind.id,
            player,
            body,
            hands,
            backpack,
        };
        self.game.universe.load_farmers(vec![farmer], id);
        self.farmers.insert(farmer_name.to_string(), farmer);
        self.containers
            .insert(format!("{}:hands", farmer_name), hands);
        self.containers
            .insert(format!("{}:backpack", farmer_name), backpack);
        self
    }

    pub fn given_farmland(mut self, farmland_key: &str, farmland_name: &str) -> Self {
        let farmland_kind = self.game.known.farmlands.find(farmland_key).unwrap();

        let space = SpaceId(self.game.physics.spaces_sequence + 1);
        let kind = self.game.known.spaces.get(farmland_kind.space).unwrap();
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

        let id = self.game.universe.farmlands_id + 1;
        let farmland = Farmland {
            id,
            kind: farmland_kind.id,
            space,
            land,
            grid,
        };
        self.game.universe.load_farmlands(vec![farmland], id);
        self.farmlands.insert(farmland_name.to_string(), farmland);
        self.current_farmland = Some(farmland);
        self
    }

    pub fn given_item(mut self, container_name: &str, item_kind: &str, item_name: &str) -> Self {
        let container = *self.containers.get(container_name).unwrap();
        let kind = self.game.known.items.find(item_kind).unwrap();
        let id = ItemId(self.game.inventory.items_sequence + 1);
        let item = Item {
            id,
            kind,
            container,
        };
        self.game.inventory.load_items(vec![item], id.0);
        self.items.insert(item_name.to_string(), id);
        self
    }

    pub fn given_drop(mut self, drop_name: &str, farmland_name: &str, cell: [usize; 2]) -> Self {
        let farmland = self.farmlands.get(farmland_name).unwrap();

        let kind = self.game.known.containers.find("<drop>").unwrap();
        let container = ContainerId(self.game.inventory.containers_sequence + 1);
        let container_component = Container {
            id: container,
            kind,
        };
        self.game
            .inventory
            .load_containers(vec![container_component], container.0);

        let kind = self.game.known.barriers.find("<drop>").unwrap();
        let barrier = BarrierId(self.game.physics.barriers_sequence + 1);
        let barrier_component = Barrier {
            id: barrier,
            kind,
            position: [0.0, 0.0],
            space: farmland.space,
        };
        self.game
            .physics
            .load_barriers(vec![barrier_component], barrier.0);

        let id = self.game.universe.drops_id + 1;
        let drop = Drop {
            id,
            container,
            barrier,
        };
        self.game.universe.load_drops(vec![drop], id);
        self.drops.insert(drop_name.to_string(), drop);
        self.containers
            .insert(drop_name.to_string(), drop.container);
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

        let kind = self.game.known.containers.find("<construction>").unwrap();
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

    pub fn when_farmer_perform(mut self, farmer_name: &str, action: Action) -> Self {
        self.current_action_result = self.game.perform_action(&farmer_name, action);
        self
    }

    pub fn then_action_events_should_be<F>(mut self, expected_events: F) -> Self
    where
        F: FnOnce(&Self) -> Vec<Event>,
    {
        assert!(self.current_action_result.is_ok());
        let actual_events =
            std::mem::replace(&mut self.current_action_result, Err(ActionError::Test)).unwrap();
        let expected_events = expected_events(&self);
        let actual_events = format!("{:?}", actual_events);
        let expected_events = format!("{:?}", expected_events);
        assert_eq!(actual_events, expected_events);
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
        unimplemented!();
        // let container_key = ContainerKey(0);
        // self.domain.known_containers.insert(
        //     container_key,
        //     Shared::new(ContainerKind {
        //         id: container_key,
        //         name: container_kind.to_string(),
        //         capacity,
        //     }),
        // );
        // self.container_kinds
        //     .insert(container_kind.to_string(), container_key);
        // self
    }

    pub fn given_container(mut self, kind: &str, container_name: &str) -> Self {
        unimplemented!();
        // let container_key = self.container_kinds.get(kind).unwrap();
        // let kind = self
        //     .domain
        //     .known_containers
        //     .get(&container_key)
        //     .unwrap()
        //     .clone();
        // let id = ContainerId(self.domain.containers_sequence + 1);
        // let container = Container { id, kind };
        // self.domain.load_containers(vec![container], id.0);
        // self.containers.insert(container_name.to_string(), id);
        // self
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
