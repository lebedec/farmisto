use crate::collections::Shared;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformKey(pub usize);

#[derive(Clone, Copy, Default, Debug)]
pub struct PlatformCell {
    pub wall: bool,
    pub inner: bool,
    pub door: bool,
    pub window: bool,
    pub shape: usize,
}
pub type Cell = [usize; 2];

pub const MAX_PLATFORMS: usize = 1000;
pub const PLATFORM_SIZE_X: usize = 120;
pub const PLATFORM_SIZE_Y: usize = 120;

pub struct PlatformKind {
    pub id: PlatformKey,
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformId(pub usize);

pub struct Platform {
    pub id: PlatformId,
    pub kind: Shared<PlatformKind>,
    pub map: [[PlatformCell; PLATFORM_SIZE_X]; PLATFORM_SIZE_Y],
    pub segments: Vec<Vec<Segment>>,
}

pub fn encode_platform_map(
    map: [[PlatformCell; PLATFORM_SIZE_X]; PLATFORM_SIZE_Y],
) -> Vec<Vec<u32>> {
    let mut data = vec![];
    for line in map {
        data.push(
            line.map(|cell| {
                let wall = if cell.wall { "1" } else { "0" };
                let inner = if cell.inner { "1" } else { "0" };
                let door = if cell.door { "1" } else { "0" };
                let window = if cell.window { "1" } else { "0" };
                [wall, inner, door, window].join("").parse().unwrap()
            })
            .to_vec(),
        );
    }
    data
}

pub fn decode_platform_map(data: Vec<Vec<u32>>) -> Vec<Vec<PlatformCell>> {
    let mut map = vec![];
    for line in data {
        map.push(
            line.iter()
                .map(|code| {
                    let mut code = code.to_string();
                    let wall = code.chars().nth(0) == Some('1');
                    let inner = code.chars().nth(1) == Some('1');
                    let door = code.chars().nth(2) == Some('1');
                    let window = code.chars().nth(3) == Some('1');

                    PlatformCell {
                        wall,
                        inner,
                        door,
                        window,
                        shape: 0,
                    }
                })
                .collect(),
        );
    }
    map
}

#[derive(Default)]
pub struct BuildingDomain {
    pub known_platforms: HashMap<PlatformKey, Shared<PlatformKind>>,
    pub platforms: Vec<Platform>,
}

pub enum Building {
    PlatformChanged {
        platform: PlatformId,
        map: [[PlatformCell; PLATFORM_SIZE_X]; PLATFORM_SIZE_Y],
    },
}

impl BuildingDomain {
    pub fn get_platform(&self, id: PlatformId) -> Option<&Platform> {
        self.platforms.iter().find(|platform| platform.id == id)
    }

    pub fn create_platform(&mut self, id: PlatformId, kind: Shared<PlatformKind>) {
        self.platforms.push(Platform {
            id,
            kind,
            map: [[PlatformCell::default(); PLATFORM_SIZE_X]; PLATFORM_SIZE_Y],
            segments: vec![],
        })
    }

    pub fn create_wall(&mut self, platform_id: PlatformId, cell: Cell) -> Vec<Building> {
        let platform = self.platforms.get_mut(platform_id.0).unwrap();
        let [cell_x, cell_y] = cell;
        platform.map[cell_y][cell_x].wall = true;

        let map = &mut platform.map;
        let mut shape_id = 0;
        let mut segment_id = 0;
        let mut segments_above = vec![vec![Segment {
            // fake segment to simplify algo
            id: segment_id,
            shape: Shape::new(RefCell::new(shape_id)),
            start: 0,
            end: PLATFORM_SIZE_X - 1,
            wall: map[0][0].wall,
        }]];
        shape_id += 1;

        for y in 0..PLATFORM_SIZE_Y {
            let mut segments = vec![];
            let segment_above = &segments_above[y][0];
            let shape = if segment_above.wall == map[y][0].wall {
                segment_above.shape.clone()
            } else {
                shape_id += 1;
                Shape::new(RefCell::new(shape_id - 1))
            };
            let mut segment = Segment {
                id: segment_id,
                shape,
                start: 0,
                end: 0,
                wall: map[y][0].wall,
            };
            segment_id += 1;
            for x in 0..PLATFORM_SIZE_X {
                if map[y][x].wall == segment.wall {
                    let segment_above = segments_above[y]
                        .iter_mut()
                        .find(|segment| x <= segment.end)
                        .unwrap();
                    if segment_above.wall == segment.wall && segment_above.shape != segment.shape {
                        //  segment_above.shape.clone_from(&segment.shape);
                    }
                    segment.end = x;
                } else {
                    let segment_above = segments_above[y]
                        .iter()
                        .find(|segment| x <= segment.end)
                        .unwrap();
                    let shape = if segment_above.wall == map[y][x].wall {
                        segment_above.shape.clone()
                    } else {
                        shape_id += 1;
                        Shape::new(RefCell::new(shape_id - 1))
                    };
                    segments.push(segment);
                    segment = Segment {
                        id: segment_id,
                        shape,
                        start: x,
                        end: x,
                        wall: map[y][x].wall,
                    };
                    segment_id += 1;
                }

                map[y][x].shape = *segment.shape.borrow();
            }
            segments.push(segment);
            segments_above.push(segments);
        }

        for x in 0..PLATFORM_SIZE_X {
            let mut should_be_inner = false;
            for y in 0..PLATFORM_SIZE_Y {
                if map[PLATFORM_SIZE_Y - y - 1][x].wall {
                    map[PLATFORM_SIZE_Y - y - 1][x].inner = should_be_inner;
                    if !should_be_inner {
                        should_be_inner = true;
                    }
                }
                if map[PLATFORM_SIZE_Y - y - 1][x].shape == 0 {
                    should_be_inner = false;
                }
            }
        }

        segments_above.remove(0); // remove fake
        platform.segments = segments_above;

        vec![Building::PlatformChanged {
            platform: platform_id,
            map: map.clone(),
        }]
    }
}

pub type Shape = Rc<RefCell<usize>>;

#[derive(Debug, Clone)]
pub struct Segment {
    pub id: usize,
    pub shape: Shape,
    pub start: usize,
    pub end: usize,
    pub wall: bool,
}
