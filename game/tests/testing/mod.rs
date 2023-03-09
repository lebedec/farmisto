use std::collections::HashMap;
use std::hash::Hash;

use plotly::color::NamedColor;
use plotly::common::Mode;
use plotly::layout::{Axis, Shape, ShapeLayer, ShapeLine, ShapeType};
use plotly::{Layout, Plot, Scatter};

use datamap::Storage;
use game::api::{Action, ActionError, Event};
use game::building::{BuildingDomain, Grid, GridId, GridKey, GridKind, Material};
use game::collections::{Dictionary, Shared};
use game::inventory::{
    Container, ContainerId, ContainerKey, ContainerKind, InventoryDomain, Item, ItemId, ItemKey,
};
use game::math::VectorMath;
use game::model::{Construction, Drop, Farmer, Farmland, Player, PlayerId, Theodolite};
use game::physics::{
    Barrier, BarrierId, BarrierKey, BarrierKind, Body, BodyId, BodyKey, BodyKind, Physics,
    PhysicsDomain, PhysicsError, Space, SpaceId, SpaceKey, SpaceKind,
};
use game::planting::{Soil, SoilId};
use game::Game;

pub fn at<T>(x: T, y: T) -> [T; 2] {
    [x, y]
}

// Some fields may contain large data, e.g. various maps.
// Asserting their content is excluded from testing because it's inefficient
// and it's better to use specific asserts.
// In most cases, it Vec<T> fields.
// So this function is needed to explicitly define these fields in tests.
pub const fn any<T>() -> Vec<T> {
    vec![]
}

pub const ANYWHERE: [usize; 2] = [0, 0];

pub struct Assertion {
    pub expected: String,
    pub actual: String,
    pub message: String,
}

pub struct GameTestScenario {
    name: String,
    plot: Plot,
    game: Game,
    debug: Option<fn(Self) -> Self>,
    farmlands: HashMap<String, Farmland>,
    farmers: HashMap<String, Farmer>,
    drops: HashMap<String, Drop>,
    theodolites: HashMap<String, Theodolite>,
    constructions: HashMap<String, Construction>,
    containers: HashMap<String, ContainerId>,
    spaces: HashMap<String, SpaceId>,
    grids: HashMap<String, GridId>,
    items: HashMap<String, ItemId>,
    current_farmland: Option<Farmland>,
    current_farmer: Option<Farmer>,
    current_action_result: Result<Vec<Event>, ActionError>,
}

impl GameTestScenario {
    pub fn new(name: String) -> Self {
        let storage = Storage::open("../assets/database.sqlite").unwrap();
        let mut game = Game::new(storage);
        game.load_game_knowledge();
        let mut plot = Plot::new();
        GameTestScenario {
            name,
            plot,
            game,
            debug: None,
            farmlands: Default::default(),
            farmers: Default::default(),
            drops: Default::default(),
            theodolites: Default::default(),
            constructions: Default::default(),
            containers: Default::default(),
            spaces: Default::default(),
            grids: Default::default(),
            items: Default::default(),
            current_farmland: None,
            current_farmer: None,
            current_action_result: Err(ActionError::Test),
        }
    }

    pub fn debug(mut self, debug: fn(Self) -> Self) -> Self {
        self.debug = Some(debug);
        self
    }

    pub fn debug_buildings(mut self, farmland: &str) -> Self {
        let farmland = self.farmland(farmland);
        let grid = self.game.building.get_grid(farmland.grid).unwrap();

        let mut layout = create_plot_layout();
        for y in 0..6 {
            for x in 0..6 {
                let cell = grid.cells[y][x];
                if cell.wall {
                    let color = if cell.marker.is_some() {
                        "#008453"
                    } else {
                        "#646464"
                    };
                    layout.add_shape(
                        Shape::new()
                            .x_ref("x")
                            .y_ref("y")
                            .shape_type(ShapeType::Rect)
                            .x0(x)
                            .y0(y)
                            .x1(x + 1)
                            .y1(y + 1)
                            .fill_color(color)
                            .opacity(0.6)
                            .layer(ShapeLayer::Above)
                            .line(ShapeLine::new().width(0.0)),
                    );
                }
            }
        }
        self.plot.set_layout(layout);
        self
    }

    pub fn present(mut self) -> Self {
        let path = format!("./tests/output/{}.html", self.name);
        create_output_directories(&path);
        self.plot.write_html(path);
        self
    }

    pub fn drop(&self, name: &str) -> Drop {
        self.drops.get(name).unwrap().clone()
    }

    pub fn theodolite(&self, name: &str) -> Theodolite {
        self.theodolites.get(name).unwrap().clone()
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

    pub fn construction(&self, name: &str) -> Construction {
        self.constructions.get(name).unwrap().clone()
    }

    pub fn farmland(&self, name: &str) -> Farmland {
        self.farmlands.get(name).unwrap().clone()
    }

    pub fn space(&self, name: &str) -> SpaceId {
        self.spaces.get(name).unwrap().clone()
    }

    pub fn grid(&self, name: &str) -> GridId {
        self.grids.get(name).unwrap().clone()
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
        let container_component = Container {
            id: hands,
            kind,
            items: vec![],
        };
        self.game
            .inventory
            .load_containers(vec![container_component], hands.0);

        let kind = self.game.known.containers.find("<backpack>").unwrap();
        let backpack = ContainerId(self.game.inventory.containers_sequence + 1);
        let container_component = Container {
            id: backpack,
            kind,
            items: vec![],
        };
        self.game
            .inventory
            .load_containers(vec![container_component], hands.0);

        let kind = self.game.known.bodies.get(farmer_kind.body).unwrap();
        let body = BodyId(self.game.physics.bodies_sequence + 1);
        let body_component = Body {
            id: body,
            kind,
            position: [0.0, 0.0],
            destination: [0.0, 0.0],
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

    pub fn given_farmland(mut self, farmland_key: &str, name: &str) -> Self {
        let farmland_kind = self.game.known.farmlands.find(farmland_key).unwrap();

        let space = SpaceId(self.game.physics.spaces_sequence + 1);
        let kind = self.game.known.spaces.get(farmland_kind.space).unwrap();
        let space_component = Space {
            id: space,
            kind,
            holes: vec![vec![0; 128]; 128],
        };
        self.game
            .physics
            .load_spaces(vec![space_component], space.0);

        let land = SoilId(self.game.planting.soils_sequence + 1);
        let kind = self
            .game
            .planting
            .known_lands
            .get(&farmland_kind.soil)
            .unwrap()
            .clone();
        let land_component = Soil {
            id: land,
            kind,
            map: vec![],
        };
        self.game.planting.load_soils(vec![land_component], land.0);

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
            cells: Grid::default_map(),
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
        self.farmlands.insert(name.to_string(), farmland);
        self.spaces.insert(name.to_string(), farmland.space);
        self.grids.insert(name.to_string(), farmland.grid);
        self.current_farmland = Some(farmland);
        self
    }

    pub fn given_theodolite(mut self, name: &str, tile: [usize; 2]) -> Self {
        let id = self.game.universe.theodolites_id + 1;
        let theodolite = Theodolite { id, cell: tile };
        self.game.universe.load_theodolites(vec![theodolite], id);
        self.theodolites.insert(name.to_string(), theodolite);
        self
    }

    pub fn given_buildings(mut self, buildings_map: &str) -> Self {
        let farmland = self.current_farmland.unwrap();
        let grid = self.game.building.get_mut_grid(farmland.grid).unwrap();
        for (y, line) in buildings_map.lines().skip(1).enumerate() {
            for (x, code) in line.trim().split_whitespace().enumerate() {
                match code {
                    "#" => {
                        grid.cells[y][x].wall = true;
                    }
                    _ => {}
                }
            }
        }
        grid.rooms = Grid::calculate_rooms(&grid.cells);
        self
    }

    pub fn given_items<const N: usize>(
        mut self,
        container_name: &str,
        item_kinds: [&str; N],
    ) -> Self {
        let container = *self.containers.get(container_name).unwrap();
        for item_kind in item_kinds {
            let kind = self.game.known.items.find(item_kind).unwrap();
            let id = ItemId(self.game.inventory.items_sequence + 1);
            let item = Item {
                id,
                kind,
                container,
            };
            self.game.inventory.load_items(vec![item], id.0);
        }
        self
    }

    pub fn given_item(mut self, item_kind: &str, item_name: &str, container_name: &str) -> Self {
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
            items: vec![],
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

    pub fn given_construction(mut self, name: &str, cell: [usize; 2]) -> Self {
        let farmland = self.current_farmland.unwrap();
        let grid = farmland.grid;

        let kind = self.game.known.containers.find("<construction>").unwrap();
        let container = ContainerId(self.game.inventory.containers_sequence + 1);
        let container_component = Container {
            id: container,
            kind,
            items: vec![],
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
        self.constructions.insert(name.to_string(), construction);
        self.containers.insert(name.to_string(), container);
        self
    }

    pub fn when_farmer_perform<F>(mut self, farmer_name: &str, action_factory: F) -> Self
    where
        F: FnOnce(&Self) -> Action,
    {
        self.current_action_result = self
            .game
            .perform_action(&farmer_name, action_factory(&self));
        self
    }

    pub fn then_events_should_be<F>(mut self, expected_events: F) -> Self
    where
        F: FnOnce(&Self) -> Vec<Event>,
    {
        let actual_events =
            std::mem::replace(&mut self.current_action_result, Err(ActionError::Test));
        let expected_events: Result<Vec<Event>, ActionError> = Ok(expected_events(&self));
        let actual_events = format!("{:?}", actual_events);
        let expected_events = format!("{:?}", expected_events);
        if let Some(debug) = self.debug {
            if actual_events != expected_events {
                self = (debug)(self);
            }
        }
        assert_eq!(actual_events, expected_events);
        self
    }

    pub fn then_action_should_fail<F>(mut self, expected_error: F) -> Self
    where
        F: FnOnce(&Self) -> ActionError,
    {
        let actual_error =
            std::mem::replace(&mut self.current_action_result, Err(ActionError::Test));
        let expected_error: Result<Vec<Event>, ActionError> = Err(expected_error(&self));
        let actual_events = format!("{:?}", actual_error);
        let expected_events = format!("{:?}", expected_error);
        if let Some(debug) = self.debug {
            if actual_events != expected_events {
                self = (debug)(self);
            }
        }
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

#[derive(Default)]
pub struct PhysicsTestScenario {
    name: String,
    plot: Plot,
    domain: PhysicsDomain,

    spaces: HashMap<String, SpaceId>,
    bodies: HashMap<String, BodyId>,
    bodies_given_position: HashMap<BodyId, [f32; 2]>,
    barriers: HashMap<String, BarrierId>,

    pub known_spaces: Dictionary<SpaceKey, SpaceKind>,
    pub known_bodies: Dictionary<BodyKey, BodyKind>,
    pub known_barriers: Dictionary<BarrierKey, BarrierKind>,

    current_events: Option<Vec<Physics>>,
    current_error: Option<PhysicsError>,
}

impl PhysicsTestScenario {
    pub fn new(name: String) -> Self {
        let mut scenario = Self::default();
        scenario.name = name;
        scenario
    }

    pub fn space(&self, name: &str) -> SpaceId {
        *self.spaces.get(name).unwrap()
    }

    pub fn body(&self, name: &str) -> BodyId {
        *self.bodies.get(name).unwrap()
    }

    pub fn barrier(&self, name: &str) -> BarrierId {
        *self.barriers.get(name).unwrap()
    }

    pub fn given_space_kind(mut self, space_kind: &str, bounds: [f32; 2]) -> Self {
        let space_key = SpaceKey(self.known_spaces.len());
        self.known_spaces.insert(
            space_key,
            space_kind.to_string(),
            SpaceKind {
                id: space_key,
                name: space_kind.to_string(),
                bounds,
            },
        );
        self
    }

    pub fn given_space(mut self, kind_name: &str, space_name: &str) -> Self {
        let id = SpaceId(self.domain.spaces_sequence + 1);
        let kind = self.known_spaces.find(kind_name).unwrap();
        let space = Space {
            id,
            kind,
            holes: vec![],
        };
        self.domain.load_spaces(vec![space], id.0);
        self.spaces.insert(space_name.to_string(), id);
        self
    }

    pub fn given_barrier_kind(mut self, barrier_kind: &str, bounds: [f32; 2]) -> Self {
        let barrier_key = BarrierKey(self.known_barriers.len());
        self.known_barriers.insert(
            barrier_key,
            barrier_kind.to_string(),
            BarrierKind {
                id: barrier_key,
                name: barrier_kind.to_string(),
                bounds,
            },
        );
        self
    }

    pub fn given_barrier(
        mut self,
        kind_name: &str,
        barrier_name: &str,
        space_name: &str,
        position: [f32; 2],
    ) -> Self {
        let space = self.space(space_name);
        let id = BarrierId(self.domain.barriers_sequence + 1);
        let kind = self.known_barriers.find(kind_name).unwrap();
        let barrier = Barrier {
            id,
            kind,
            position,
            space,
        };
        self.domain.load_barriers(vec![barrier], id.0);
        self.barriers.insert(barrier_name.to_string(), id);
        self
    }

    pub fn given_body_kind(mut self, body_kind: &str, speed: f32, radius: f32) -> Self {
        let body_key = BodyKey(self.known_bodies.len());
        self.known_bodies.insert(
            body_key,
            body_kind.to_string(),
            BodyKind {
                id: body_key,
                name: body_kind.to_string(),
                speed,
                radius,
            },
        );
        self
    }

    pub fn given_body(
        mut self,
        kind_name: &str,
        body_name: &str,
        space_name: &str,
        position: [f32; 2],
    ) -> Self {
        let space = self.space(space_name);
        let id = BodyId(self.domain.bodies_sequence + 1);
        let kind = self.known_bodies.find(kind_name).unwrap();
        let body = Body {
            id,
            kind,
            position,
            destination: position,
            space,
        };
        self.domain.load_bodies(vec![body], id.0);
        self.bodies.insert(body_name.to_string(), id);
        self.bodies_given_position.insert(id, position);
        self
    }

    pub fn when_create_barrier(
        mut self,
        kind: &str,
        name: &str,
        space: &str,
        position: [f32; 2],
    ) -> Self {
        let space = self.space(space);
        let kind = self.known_barriers.find(kind).unwrap();
        match self.domain.create_barrier(space, kind, position, false) {
            Ok((barrier, operation)) => {
                let events = operation();
                self.current_error = None;
                self.current_events = Some(events);
                self.barriers.insert(name.to_string(), barrier);
            }
            Err(error) => {
                self.current_error = Some(error);
                self.current_events = None;
            }
        }
        self
    }

    pub fn when_move_body(mut self, body: &str, direction: [f32; 2]) -> Self {
        let body = self.body(body);
        match self.domain.move_body2(body, direction) {
            Ok(_) => {
                self.current_error = None;
                self.current_events = Some(vec![]);
            }
            Err(error) => {
                self.current_error = Some(error);
                self.current_events = None;
            }
        }
        self
    }

    pub fn when_update(mut self, iterations: usize, time: f32) -> Self {
        let mut events = vec![];
        for _ in 0..iterations {
            let iteration_events = self.domain.update(time);
            events.extend(iteration_events);
        }
        self.current_events = Some(events);
        self
    }

    pub fn then_error<F>(mut self, error_factory: F) -> Self
    where
        F: FnOnce(&Self) -> PhysicsError,
    {
        let expected_error = Some(error_factory(&self));
        let expected_error = format!("{:?}", expected_error);
        let actual_error = format!("{:?}", self.current_error);
        assert_eq!(expected_error, actual_error);
        self
    }

    pub fn then_events<F, D>(mut self, events_factory: F, debug: D) -> Self
    where
        F: FnOnce(&Self) -> Vec<Physics>,
        D: FnOnce(Self) -> Self,
    {
        let expected_events = Some(events_factory(&self));
        let expected_events = format!("{:?}", expected_events);
        let actual_events = format!("{:?}", self.current_events);
        if expected_events != actual_events {
            self = debug(self)
        }
        assert_eq!(expected_events, actual_events);
        self
    }

    pub fn present(mut self) -> Self {
        let path = format!("./tests/output/{}.html", self.name);
        create_output_directories(&path);
        self.plot.write_html(path);
        self
    }

    pub fn debug_body_movement(mut self, body_name: &str, shape_shown: bool) -> Self {
        let body = self.body(body_name);
        let given_position = self.bodies_given_position.get(&body).unwrap();
        let mut x = vec![given_position[0]];
        let mut y = vec![given_position[1]];

        if let Some(events) = self.current_events.as_ref() {
            for event in events {
                if let Physics::BodyPositionChanged { id, position, .. } = event {
                    if id == &body {
                        x.push(position[0]);
                        y.push(position[1]);
                    }
                }
            }
        }
        let trace = Scatter::new(x.clone(), y.clone()).name(body_name);
        self.plot.add_trace(trace);

        self
    }

    pub fn debug_space(mut self, space: &str) -> Self {
        let space = self.space(space);
        let mut layout = create_plot_layout();
        for barrier in &self.domain.barriers[space.0] {
            let offset = barrier.kind.bounds.mul(0.5);
            let min = barrier.position.sub(offset);
            let max = barrier.position.add(offset);
            layout.add_shape(
                Shape::new()
                    .x_ref("x")
                    .y_ref("y")
                    .shape_type(ShapeType::Rect)
                    .x0(min[0])
                    .y0(min[1])
                    .x1(max[0])
                    .y1(max[1])
                    .fill_color("#646464")
                    .opacity(0.6)
                    .layer(ShapeLayer::Above)
                    .line(ShapeLine::new().width(0.0)),
            );
        }
        for body in &self.domain.bodies[space.0] {
            layout.add_shape(
                Shape::new()
                    .x_ref("x")
                    .y_ref("y")
                    .shape_type(ShapeType::Circle)
                    .x0(body.position[0] - 0.5)
                    .y0(body.position[1] - 0.5)
                    .x1(body.position[0] + 0.5)
                    .y1(body.position[1] + 0.5)
                    .fill_color("#32cbfe")
                    .opacity(0.6)
                    .layer(ShapeLayer::Above)
                    .line(ShapeLine::new().width(0.0)),
            );
        }
        self.plot.set_layout(layout);
        self
    }
}

fn create_plot_layout() -> Layout {
    let x_axis = Axis::new()
        .range(vec![0.0, 6.0])
        .auto_margin(true)
        .zero_line(false);
    let y_axis = Axis::new()
        .range(vec![0.0, 6.0])
        .auto_margin(true)
        .zero_line(false)
        .overlaying("x");
    Layout::new()
        .x_axis(x_axis)
        .y_axis(y_axis)
        .width(512)
        .height(512)
        .auto_size(false)
}

fn create_output_directories(path: &str) {
    let path = std::path::Path::new(path);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
}

#[macro_export]
macro_rules! scenario {
    () => {{
        fn _f() {}
        fn _type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = _type_name_of(_f);
        (&name[..name.len() - 4]).replace("::", "/").to_string()
    }};
}

#[macro_export]
macro_rules! events {
    () => (
        vec![]
    );
    ($($x:expr,)*) => (vec![$($x.into()),*])
}
