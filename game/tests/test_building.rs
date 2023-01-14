use game::building::{
    BuildingDomain, Platform, PlatformCell, PlatformId, PlatformKey, PlatformKind, Shape,
};
use game::collections::Shared;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[test]
fn test_something() {
    let mut map = Platform::default_map();
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
            map[y][x] = PlatformCell {
                wall: code == "#",
                inner: false,
                door: false,
                window: false,
            };
        }
    }

    let t1 = Instant::now();
    let shapes = Platform::calculate_shapes(&map);

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
            (shape.id != Shape::EXTERIOR_ID && !shape.contour),
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
                let cell = 1 << (Platform::SIZE_X - x - 1);
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . . . . . . .
            . # # # # # .
            . # . . . . .
            . # . . . . .
            . . . . . . .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . # # . # # .
            . # . . . # .
            . . . . . . .
            . # . . . # .
            . # # . # # .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . # # # # # .
            . # . . . # .
            . # . . . # .
            . # . . . # .
            . # # . # # .
            . . . . . . .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
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
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . . . . . . . . .
            . # # # # # # # .
            . # . . . . . # .
            . # . # # # . # .
            . # # # . # # # .
            . . . . . . . . .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . # # # # # .
            . # . . . # .
            . # . . . # .
            . # # # # # .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
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
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
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
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . # # # # # # .
            . # . . # . # .
            . # . . # . # .
            . # . . . . # .
            . # # # # # # .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . # # # # # # .
            . # . . . . # .
            . # . # . . # .
            . # . # . . # .
            . # # # # # # .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
        .given_platform_kind("regular")
        .given_platform("regular", "platform")
        .when_player_builds_on(
            "platform",
            r#"
            . # # # # # # # .
            . # . . . # . # .
            . # . . . # . # .
            . # # # . . . # .
            . # . . . . . # .
            . # # # # # # # .
            "#,
        )
        .then_platform_shapes_should_be(
            "platform",
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
    platforms: HashMap<String, PlatformId>,
    platform_kinds: HashMap<String, PlatformKey>,
}

impl BuildingTestScenario {
    pub fn new() -> Self {
        Self {
            domain: BuildingDomain::default(),
            platforms: Default::default(),
            platform_kinds: Default::default(),
        }
    }

    pub fn given_platform_kind(mut self, platform_kind: &str) -> Self {
        let platform_key = PlatformKey(0);
        self.domain.known_platforms.insert(
            platform_key,
            Shared::new(PlatformKind {
                id: platform_key,
                name: platform_kind.to_string(),
            }),
        );
        self.platform_kinds
            .insert(platform_kind.to_string(), platform_key);
        self
    }

    pub fn given_platform(mut self, kind: &str, platform: &str) -> Self {
        let platform_id = PlatformId(0);
        let platform_key = self.platform_kinds.get(kind).unwrap();
        self.domain.create_platform(
            platform_id,
            self.domain
                .known_platforms
                .get(&platform_key)
                .unwrap()
                .clone(),
        );
        self.platforms.insert(platform.to_string(), platform_id);
        self
    }

    pub fn when_player_builds_on(mut self, platform: &str, building_map: &str) -> Self {
        let platform_id = self.platforms.get(platform).unwrap();
        for (y, line) in building_map.lines().skip(1).enumerate() {
            for (x, code) in line.trim().split_whitespace().enumerate() {
                match code {
                    "#" => {
                        self.domain.create_wall(*platform_id, [x, y]);
                    }
                    _ => {}
                }
            }
        }
        self
    }

    pub fn then_platform_shapes_should_be(self, platform: &str, expected: &str) -> Self {
        // let expected = expected
        //     .lines()
        //     .map(|line| line.trim())
        //     .filter(|line| !line.is_empty())
        //     .collect::<Vec<&str>>()
        //     .join("\n");
        //
        // let expected_y = expected.lines().count();
        // let expected_x = expected.lines().nth(0).unwrap().split_whitespace().count();
        //
        // let platform_id = self.platforms.get(platform).unwrap();
        // let platform = &self.domain.platforms[platform_id.0];
        //
        // let mut actual = vec![];
        // for (y, segments) in platform.segments.iter().take(expected_y).enumerate() {
        //     let mut line = vec![];
        //     for x in 0..expected_x {
        //         let code = match platform.map[y][x].shape {
        //             0 => ".".to_string(),
        //             shape => {
        //                 if platform.map[y][x].wall {
        //                     let code = "#abcdefghijklmnopqrstuvwxyz".chars().nth(shape).unwrap();
        //                     if platform.map[y][x].inner {
        //                         code.to_string()
        //                     } else {
        //                         code.to_uppercase().to_string()
        //                     }
        //                 } else {
        //                     shape.to_string()
        //                 }
        //             }
        //         };
        //         line.push(code);
        //     }
        //     // for segment in segments {
        //     //     let code = match *segment.shape.borrow() {
        //     //         0 => ".".to_string(),
        //     //         shape => {
        //     //             if segment.wall {
        //     //                 let code = "#abcdefghijklmnopqrstuvwxyz".chars().nth(shape).unwrap();
        //     //                 code.to_string()
        //     //             } else {
        //     //                 shape.to_string()
        //     //             }
        //     //         }
        //     //     };
        //     //     let length = 1 + segment.end - segment.start;
        //     //     let length = length.min(expected_x - line.len());
        //     //     line.extend(vec![code; length]);
        //     //     if segment.end >= expected_x {
        //     //         break;
        //     //     }
        //     // }
        //     actual.push(line.join(" "));
        // }
        // let actual = actual.join("\n");
        //
        // assert_eq!(actual, expected, "actual shapes: \n{}", actual);
        self
    }
}
