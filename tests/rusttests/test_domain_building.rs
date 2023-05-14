use crate::testing::BuildingTestScenario;
use game::building::{Cell, Grid, Room};
use std::time::Instant;

mod testing;

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
                marker: false,
                material: Default::default(),
            };
        }
    }

    let t1 = Instant::now();
    let shapes = Grid::calculate_rooms(&map);

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
            shape.area_y,
            shape.area.len(),
        );
        for y in 0..def_y {
            let row = if y >= shape.area_y && y - shape.area_y < shape.area.len() {
                shape.area[y - shape.area_y]
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
