use game::building::{
    BuildingDomain, PlatformId, PlatformKey, PlatformKind, PLATFORM_SIZE_X, PLATFORM_SIZE_Y,
};
use game::collections::Shared;
use std::collections::HashMap;
use std::ptr;
use std::time::Instant;

#[derive(Debug)]
struct Shape {
    pub id: usize,
    pub contour: bool,
    pub y: usize,
    pub rows: Vec<u128>,
    pub active: bool,
}

pub fn detect_touch(row: u128, shapes: Vec<u128>) -> Vec<u128> {
    let mut appends = vec![0u128; shapes.len()];
    let mut value: u128 = row;
    let mut i = 0;
    while value != 0 {
        let skip_zeros = value.leading_zeros();
        i += skip_zeros;
        value = value << skip_zeros;
        // println!("skip to {}", i);
        let width = value.leading_ones();
        let val = u128::MAX >> (128 - width);
        let segment = val << (128 - i - width);
        // println!("segment {}..{} {:#010b}", i, i + width - 1, segment);
        let mut any = false;
        for (index, append) in appends.iter_mut().enumerate() {
            if index < shapes.len() && shapes[index] & segment != 0 {
                *append = *append | segment;
                any = true;
                continue;
            }
        }
        if !any {
            appends.push(segment);
        }
        i += width;
        if width == 128 {
            break;
        }
        value = value << width;
    }
    appends
}

#[test]
fn test_something() {
    println!("value {} {:#016b}", 5, 5);
    println!("value {} {:#016b}", 0b0101, 0b0101);
    println!("value {} {:#016b}", 1 << 0 | 1 << 2, 1 << 0 | 1 << 2);
    println!("value {} {:#016b}", u16::MAX >> 13, u16::MAX >> 13);
    println!("value {} {:#016b}", 3 & 2, 3 & 2);
    println!("value {} {:#016b}", 3 & 4, 3 & 4);

    // 01100111000
    // 00111100000
    //   1  1

    const MAP_SIZE_X: usize = 128;
    const MAP_SIZE_Y: usize = 128;
    let mut map = [[0u8; MAP_SIZE_X]; MAP_SIZE_Y];
    // let def_map = r#"
    // . . . . . . . . . .
    // . . # # # # # # # .
    // . . # . . . . . # .
    // . . # . . . . . # .
    // . . # # # # # # # .
    // . . . . . . . . . .
    // "#;
    let def_map = r#"
    . . . . . . . . . . . . . . . .
    . . # # # # # # # . . . . . . .
    . . # . . . . . # . . . . . . .
    . . # . # # # . # . # # # . . .
    . . # . # . # . # . . . # . . .
    . . # . # # # # # . . . # . . .
    . . # . . . . . # . # . # . . .
    . . # # # # # # # . # # # . . .
    . . . . . . . . . . . . . . . .
    "#;
    let def_map = r#"
    . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
    . . # # # # # # # . . . . . . # # # # # # . . . . . . . . . . .
    . . # . . . . . # . . . . . . # . . . . # . . . . . . . . . . .
    . . # . # # # . # . # # # . . # # # . . # . . . . . . . . . . .
    . . # . # . # . # . . . # . . . . # . . # . . . . . . . . . . .
    . . # . # . # . # . . . # . . . . # . . # . . . . . . . . . . .
    . . # . . . . . # . # . # . . . . # . . # . . . . . . . . . . .
    . . # # # # # # # . # # # . . . . # . . # . . . . . . . . . . .
    . . . . . . . . . . . . . . . . . # . . # . . . . . . . . . . .
    . # # # . # # # # # # . . . . . . # . . # . . . . . . . . . . .
    . # . # . # . . . . # . . . . . . # . . # . . . . . . . . . . .
    . # . # # # . . # # # . . . . . . # . . # . . . . . . . . . . .
    . # . . . . . . # . . . . . . . . # . . # . . . . . . . . . . .
    . # # . # # # . # . # # # # # # . # . . # . . . . . . . . . . .
    . . # . # . # . # . . . . . . # . # . . # . . . . . . . . . . .
    . . # . # # # # # . # # # # . # . # . . # . . . . . . . . . . .
    . . # . . . . . # . # . . # . # . # . . # . . . . . . . . . . .
    . . # # # # # # # . # # # # # # . # . . # . . . . . . . . . . .
    . . . . . . . . . . . . . . . . . # . . # . . . . . . . . . . .
    . . . . . . . # # # # # # # # # # # . . # # # . . . . . . . . .
    . . . . . . . # . . . . . . . . . . . . . . # . . . . . . . . .
    . . . . . . . # # # # # # # # # # # # # # # # . . . . . . . . .
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
            map[y][x] = match code {
                "#" => 1,
                _ => 0,
            };
        }
    }

    let t1 = Instant::now();

    let exterior_shape = Shape {
        id: 0,
        contour: false,
        y: 0,
        rows: vec![u128::MAX],
        active: true,
    };
    let mut shapes: Vec<Shape> = vec![exterior_shape];
    for y in 1..MAP_SIZE_Y {
        let mut row = 0;
        for x in 0..MAP_SIZE_X {
            let cell = 1u128 << (MAP_SIZE_X - x - 1);
            let contour = map[y][x] == 1;
            if !contour {
                row = row | cell;
            }
        }

        let candidates: Vec<u128> = shapes
            .iter()
            .map(|shape| match shape.active {
                true => *shape.rows.last().unwrap(),
                false => 0,
            })
            .collect();
        let candidates_n = candidates.len();
        let touches = detect_touch(row, candidates);

        for i in 0..touches.len() {
            if i >= candidates_n {
                shapes.push(Shape {
                    id: shapes.len(),
                    contour: false,
                    y,
                    rows: vec![touches[i]],
                    active: true,
                });
            } else {
                if touches[i] != 0 {
                    shapes[i].rows.push(touches[i]);
                } else {
                    shapes[i].active = false;
                }
            }
        }

        // let mut shape_ptr = None;
        // for x in 0..MAP_SIZE_X {
        //     let cell = 1 << (MAP_SIZE_X - x - 1);
        //     let contour = map[y][x] == 1;
        //     //println!("Y: {}, X: {}, shapes: {:?}", y, x, shapes);
        //     let shape_above = shapes
        //         .iter_mut()
        //         .position(|shape| {
        //             shape.max_y + 1 == y && shape.rows[shape.rows.len() - 1] & cell == cell
        //         })
        //         .unwrap();
        //
        //     if shape_ptr.is_none() {
        //         if shapes[shape_above].contour == contour {
        //             shape_ptr = Some(shape_above);
        //         } else {
        //             let shape = Shape {
        //                 id: shapes.len(),
        //                 contour,
        //                 y,
        //                 max_y: y - 1,
        //                 rows: vec![],
        //                 current: cell,
        //             };
        //             shapes.push(shape);
        //             shape_ptr = Some(shapes.len() - 1);
        //         }
        //     } else {
        //         if shapes[shape_ptr.unwrap()].contour == contour {
        //             if shapes[shape_above].contour == contour && shape_above != shape_ptr.unwrap() {
        //                 let t1 = Instant::now();
        //                 let source = shapes.remove(shape_above);
        //                 if shape_ptr.unwrap() > shape_above {
        //                     shape_ptr = Some(shape_ptr.unwrap() - 1);
        //                 }
        //
        //                 let shape = &mut shapes[shape_ptr.unwrap()];
        //                 let offset = source.y as isize - shape.y as isize;
        //                 if offset < 0 {
        //                     shape.y = source.y;
        //                     let mut rows = vec![0; offset.abs() as usize];
        //                     rows.extend(&shape.rows);
        //                     shape.rows = rows;
        //                 }
        //                 for (index, row) in source.rows.into_iter().enumerate() {
        //                     shape.rows[index] = shape.rows[index] | row;
        //                 }
        //             } else {
        //                 // normal
        //             }
        //         } else {
        //             if shapes[shape_above].contour == contour {
        //                 shape_ptr = Some(shape_above);
        //             } else {
        //                 let shape = Shape {
        //                     id: shapes.len(),
        //                     contour,
        //                     y,
        //                     max_y: y - 1,
        //                     rows: vec![],
        //                     current: cell,
        //                 };
        //                 shapes.push(shape);
        //                 shape_ptr = Some(shapes.len() - 1);
        //             }
        //         }
        //     }
        //
        //     let shape = &mut shapes[shape_ptr.unwrap()];
        //     shape.current = shape.current | cell;
        // }
        //
        // for mut shape in shapes.iter_mut() {
        //     if shape.current != 0 {
        //         shape.rows.push(shape.current);
        //         shape.max_y = y;
        //         shape.current = 0;
        //     }
        // }
    }

    println!("elapsed: {}", t1.elapsed().as_secs_f64());
    println!("shapes: {:?}", shapes.len());
    for shape in shapes {
        // if shape.id == 0 {
        //     continue;
        // }
        println!(
            "shape {} contour: {} interior: {} (y:{} rows:{})",
            shape.id,
            shape.contour,
            (shape.id != 0 && !shape.contour),
            shape.y,
            shape.rows.len()
        );
        for y in 0..def_y {
            let row = if y >= shape.y && y - shape.y < shape.rows.len() {
                shape.rows[y - shape.y]
            } else {
                0
            };
            for x in 0..def_x {
                let cell = 1 << (MAP_SIZE_X - x - 1);
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
        let expected = expected
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join("\n");

        let expected_y = expected.lines().count();
        let expected_x = expected.lines().nth(0).unwrap().split_whitespace().count();

        let platform_id = self.platforms.get(platform).unwrap();
        let platform = &self.domain.platforms[platform_id.0];

        let mut actual = vec![];
        for (y, segments) in platform.segments.iter().take(expected_y).enumerate() {
            let mut line = vec![];
            for x in 0..expected_x {
                let code = match platform.map[y][x].shape {
                    0 => ".".to_string(),
                    shape => {
                        if platform.map[y][x].wall {
                            let code = "#abcdefghijklmnopqrstuvwxyz".chars().nth(shape).unwrap();
                            if platform.map[y][x].inner {
                                code.to_string()
                            } else {
                                code.to_uppercase().to_string()
                            }
                        } else {
                            shape.to_string()
                        }
                    }
                };
                line.push(code);
            }
            // for segment in segments {
            //     let code = match *segment.shape.borrow() {
            //         0 => ".".to_string(),
            //         shape => {
            //             if segment.wall {
            //                 let code = "#abcdefghijklmnopqrstuvwxyz".chars().nth(shape).unwrap();
            //                 code.to_string()
            //             } else {
            //                 shape.to_string()
            //             }
            //         }
            //     };
            //     let length = 1 + segment.end - segment.start;
            //     let length = length.min(expected_x - line.len());
            //     line.extend(vec![code; length]);
            //     if segment.end >= expected_x {
            //         break;
            //     }
            // }
            actual.push(line.join(" "));
        }
        let actual = actual.join("\n");

        assert_eq!(actual, expected, "actual shapes: \n{}", actual);
        self
    }
}
