use game::building::{BuildingDomain, Cell, Grid, GridId, GridKey, GridKind, Material, Room};
use game::collections::Shared;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[test]
fn test_something() {
    let mut map = Grid::default_map();
    let def_map = r#"
    . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
    . . # # # # # # # . . . . . . # # # # # # . . . . . . . . . . .
    . . # . . . . . # . . . . . . # . . . . # # # # # . . . . . . .
    . . # . # # # . # . # # # . . # # # . . # # . . # . . . . . . .
    . . # . # . # . # . . . # . . . . # . . # # . . # . . . . . . .
    . . # . # . # . # . . . # . . . . # . . # # # # # . . . . . . .
    . . # . . . . . # . # . # . . . . # . . # # . . . . . . . . . .
    . . # # # # # # # . # # # . . . . # . . # # . . . . . . . . . .
    . . . . . . . . . . . . . . . . . # . . # # # # # # # . . . . .
    . # # # . # # # # # # . . . . . . # . . # . . # . . # . . . . .
    . # . # . # . . . . # . . . . . . # . . # . . # . . # . . . . .
    . # . # # # . . # # # . . . . . . # . . # # # # # # # . . . . .
    . # . . . . . . # . . . . . . . . # . . # . . . . . # . . . . .
    . # # . # # # . # . # # # # # # . # . . # . . . . . # . . . . .
    . . # . # . # . # . . . . . . # . # . . # # # # # # # . . . . .
    . . # . # # # # # . # # # # . # . # . . # . . . . . . . . . . .
    . . # . . . . . # . # . . # . # . # . . # . . . . . . . . . . .
    . . # # # # # # # . # # # # # # . # . . # # # # # # # # # . . .
    . . . . . . . . . . . . . . . . . # . . . . . . . . . . # . . .
    . . . . . . . # # # # # # # # # # # . . . . . . . . . . # . . .
    . . . . . . . # . . . . . . . . . . . . . # # # # # . . # . . .
    . . . . . . . # . . . . . . . . . . . . . # . . . # . . # . . .
    . . . . . . . # . # # # # # # # . . . . . # . . . # . . # . . .
    . . . . . . . # . # . . . . . # . . . . . # . . . # . . # . . .
    . . . . . . . # . # . # # # . # . . . . . # # # # # . . # . . .
    . . . . . . . # . # . # . # . # . . . . . . . . . . . . # . . .
    . . . . . . . # . # . # # # . # . . . . . . . . . . . . # . . .
    . . . . . . . # . # . . . . . # . . # # # # # # # # # # # . . .
    . . . . . . . # . # # # # # # # . . # . . . . . . . . . . . . .
    . . . . . . . # . . . . . . . . . . # . . . . . . . . . . . . .
    . . . . . . . # # # # # # # # # # # # . . . . . . . . . . . . .
    . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
    "#;
    let def_y = def_map.lines().skip(1).count();
    let def_x = def_map
        .lines()
        .skip(1)
        .nth(0)
        .unwrap()
        .trim()
        .split_whitespace()
        .count();
    for (y, line) in def_map.lines().skip(1).enumerate() {
        for (x, code) in line.trim().split_whitespace().enumerate() {
            map[y][x] = Cell {
                wall: code == "#",
                inner: false,
                door: false,
                window: false,
                material: Default::default(),
            };
        }
    }

    let t1 = Instant::now();
    let shapes = Grid::calculate_shapes(&map);

    println!("elapsed: {}", t1.elapsed().as_secs_f64());
    println!("shapes: {:?}", shapes.len());
    for shape in shapes {
        // if shape.id == 0 {
        //     continue;
        // }
        println!(
            "shape {} contour:{} interior:{} y:{} rows:{}",
            shape.id,
            shape.contour,
            (shape.id != Room::EXTERIOR_ID && !shape.contour),
            shape.rows_y,
            shape.rows.len(),
        );
        for y in 0..def_y {
            let row = if y >= shape.rows_y && y - shape.rows_y < shape.rows.len() {
                shape.rows[y - shape.rows_y]
            } else {
                0
            };

            for x in 0..def_x {
                let cell = 1 << (Grid::COLUMNS - x - 1);
                let code = if row & cell == cell { "." } else { "#" };
                print!(" {}", code);
            }
            println!();
        }
    }
}

#[test]
fn test_shapes_in_left_top_corner() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . . . . . . .
            . # # # # # .
            . # . . . . .
            . # . . . . .
            . . . . . . .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . . . . . . .
            . a A A A A .
            . a 2 2 2 2 2
            . A 2 2 2 2 2
            . . . . . . .
            "#,
        );
}

#[test]
fn test_shapes_in_corners() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # . # # .
            . # . . . # .
            . . . . . . .
            . # . . . # .
            . # # . # # .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . a A . B b .
            . A 3 3 3 B .
            . . . . . . .
            . d . . . e .
            . D D . F F .
            "#,
        );
}

#[test]
fn test_shapes_in_incomplete_room() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # # # # .
            . # . . . # .
            . # . . . # .
            . # . . . # .
            . # # . # # .
            . . . . . . .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . 1 1 1 1 1 .
            . 1 . . . 1 .
            . 1 . . . 1 .
            . 1 . . . 1 .
            . 1 1 . 1 1 .
            . . . . . . .
            "#,
        );
}

#[test]
fn test_shapes_in_incomplete_complex_room() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . . . . . . . . . .
            . # # # # # . . . .
            . # . . . # # # # .
            . # . . . . . . # .
            . # . # # # . . # .
            . # # # . # # . # .
            . . . . . . . . . .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . . . . . . . . . .
            . 1 1 1 1 1 . . . .
            . 1 . . . 1 1 1 1 .
            . 1 . . . . . . 1 .
            . 1 . 1 1 1 . . 1 .
            . 1 1 1 . 1 1 . 1 .
            . . . . . . . . . .
            "#,
        );
}

#[test]
fn test_shapes_in_non_convex_room() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . . . . . . . . .
            . # # # # # # # .
            . # . . . . . # .
            . # . # # # . # .
            . # # # . # # # .
            . . . . . . . . .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . . . . . . . . .
            . 1 1 1 1 1 1 1 .
            . 1 . . . . . 1 .
            . 1 . 1 1 1 . 1 .
            . 1 1 1 . 1 1 1 .
            . . . . . . . . .
            "#,
        );
}

#[test]
fn test_shapes_in_rectangle_room() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # # # # .
            . # . . . # .
            . # . . . # .
            . # # # # # .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . 1 1 1 1 1 .
            . 1 2 2 2 1 .
            . 1 2 2 2 1 .
            . 1 1 1 1 1 .
            "#,
        );
}

#[test]
fn test_shapes_in_inner_room() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # # # # # # # .
            . # . . . . . . # .
            . # . # # # # . # .
            . # . # . . # . # .
            . # . # # # # . # .
            . # . . . . . . # .
            . # # # # # # # # .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . a a a a a a a a .
            . a 2 2 2 2 2 2 a .
            . a 2 b b b b 2 a .
            . a 2 b 4 4 b 2 a .
            . a 2 b b b b 2 a .
            . a 2 2 2 2 2 2 a .
            . A A A A A A A A .
            "#,
        );
}

#[test]
fn test_shapes_in_buildings_enter_each_other() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . . . . . . . . . .
            . # # # # # # # . .
            . # . . . . . # . .
            . # . # # # # # . .
            . # . # . . . . . .
            . # . # . # # # # .
            . # . # . # . . # .
            . # . # . # # # # .
            . # . # . . . . . .
            . # . # # # # . . .
            . # . . . . # . . .
            . # # # # # # . . .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . . . . . . . . . .
            . a a a a a a a . .
            . a 1 1 1 1 1 a . .
            . a 1 a A A A A . .
            . a 1 a . . . . . .
            . a 1 a . b b b b .
            . a 1 a . b 2 2 b .
            . a 1 a . B B B B .
            . a 1 a . . . . . .
            . a 1 a a a a . . .
            . a 1 1 1 1 a . . .
            . A A A A A A . . .
            "#,
        );
}

#[test]
fn test_shapes_in_room_with_top_division() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # # # # # .
            . # . . # . # .
            . # . . # . # .
            . # . . . . # .
            . # # # # # # .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . 1 1 1 1 1 1 .
            . 1 2 2 1 2 1 .
            . 1 2 2 1 2 1 .
            . 1 2 2 2 2 1 .
            . 1 1 1 1 1 1 .
            "#,
        );
}

#[test]
fn test_shapes_in_room_with_bottom_division() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # # # # # .
            . # . . . . # .
            . # . # . . # .
            . # . # . . # .
            . # # # # # # .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . 1 1 1 1 1 1 .
            . 1 2 2 2 2 1 .
            . 1 2 1 2 2 1 .
            . 1 2 1 2 2 1 .
            . 1 1 1 1 1 1 .
            "#,
        );
}

#[test]
fn test_shapes_in_room_two_divisions() {
    BuildingTestScenario::new()
        .given_grid_kind("regular")
        .given_grid("regular", "grid")
        .when_player_builds_on(
            "grid",
            r#"
            . # # # # # # # .
            . # . . . # . # .
            . # . . . # . # .
            . # # # . . . # .
            . # . . . . . # .
            . # # # # # # # .
            "#,
        )
        .then_grid_rooms_should_be(
            "grid",
            r#"
            . 1 1 1 1 1 1 1 .
            . 1 2 2 2 1 2 1 .
            . 1 2 2 2 1 2 1 .
            . 1 1 1 2 2 2 1 .
            . 1 2 2 2 2 2 1 .
            . 1 1 1 1 1 1 1 .
            "#,
        );
}

struct BuildingTestScenario {
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
        let grid_id = GridId(0);
        let grid_key = self.grid_kinds.get(kind).unwrap();
        self.domain.create_grid(
            grid_id,
            self.domain.known_grids.get(&grid_key).unwrap().clone(),
        );
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

    pub fn then_grid_rooms_should_be(self, grid: &str, expected: &str) -> Self {
        self
    }
}
