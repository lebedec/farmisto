use crate::collections::Shared;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformKey(pub usize);

#[derive(Clone, Copy, Default, Debug)]
pub struct PlatformCell {
    pub wall: bool,
    pub inner: bool,
    pub door: bool,
    pub window: bool,
}

pub type Cell = [usize; 2];

pub struct PlatformKind {
    pub id: PlatformKey,
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformId(pub usize);

#[derive(Debug, Default, Clone, bincode::Encode, bincode::Decode)]
pub struct Shape {
    pub id: usize,
    pub contour: bool,
    pub rows_y: usize,
    pub rows: Vec<u128>,
    pub active: bool,
}

impl Shape {
    pub const EXTERIOR_ID: usize = 0;
}

pub struct Platform {
    pub id: PlatformId,
    pub kind: Shared<PlatformKind>,
    pub map: [[PlatformCell; Platform::SIZE_X]; Platform::SIZE_Y],
    pub shapes: Vec<Shape>,
}

#[derive(Default)]
pub struct BuildingDomain {
    pub known_platforms: HashMap<PlatformKey, Shared<PlatformKind>>,
    pub platforms: Vec<Platform>,
}

pub enum Building {
    PlatformChanged {
        platform: PlatformId,
        map: [[PlatformCell; Platform::SIZE_X]; Platform::SIZE_Y],
        shapes: Vec<Shape>,
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
            map: [[PlatformCell::default(); Platform::SIZE_X]; Platform::SIZE_Y],
            shapes: vec![],
        })
    }

    pub fn create_wall(&mut self, platform_id: PlatformId, cell: Cell) -> Vec<Building> {
        let platform = self.platforms.get_mut(platform_id.0).unwrap();
        let [cell_x, cell_y] = cell;
        platform.map[cell_y][cell_x].wall = true;
        platform.shapes = Platform::calculate_shapes(&platform.map);
        vec![Building::PlatformChanged {
            platform: platform_id,
            map: platform.map.clone(),
            shapes: platform.shapes.clone(),
        }]
    }
}

impl Platform {
    pub const SIZE_X: usize = 128;
    pub const SIZE_Y: usize = 128;

    pub fn default_map() -> [[PlatformCell; Platform::SIZE_X]; Platform::SIZE_Y] {
        [[PlatformCell::default(); Platform::SIZE_X]; Platform::SIZE_Y]
    }

    fn grow_shapes(shapes: &mut Vec<Shape>) {
        for shape in shapes {
            if shape.id == Shape::EXTERIOR_ID {
                continue;
            }
            // grow vertically
            let mut area = vec![0; shape.rows.len() + 2];
            for i in 1..=shape.rows.len() {
                area[i - 1] |= shape.rows[i - 1];
                area[i] |= shape.rows[i - 1];
                area[i + 1] |= shape.rows[i - 1];
            }
            // grow horizontally by segments
            for row in area.iter_mut() {
                let mut value: u128 = *row;
                let mut i = 0;
                let mut grow_row = 0;
                while value != 0 {
                    let skip_zeros = value.leading_zeros();
                    i += skip_zeros;
                    value = value << skip_zeros;

                    let width = value.leading_ones() + 2;
                    i -= 1;
                    let val = u128::MAX >> (128 - width);
                    let segment = val << (128 - i - width);
                    grow_row = grow_row | segment;

                    i += width - 1;
                    if width == 128 {
                        break;
                    }
                    value = value << (width - 2);
                }
                *row = grow_row;
            }
            shape.rows_y = shape.rows_y - 1;
            shape.rows = area;
        }
    }

    fn merge_shapes(merges: Vec<[usize; 2]>, mut shapes: Vec<Shape>) -> Vec<Shape> {
        if !merges.is_empty() {
            let mut to_delete = vec![];
            for [source, destination] in merges {
                to_delete.push(source);
                let source = &mut shapes[source];
                source.active = false;
                let source_y = source.rows_y;
                let source_rows = source.rows.clone();
                let shape = &mut shapes[destination];

                let offset = source_y as isize - shape.rows_y as isize;
                if offset < 0 {
                    shape.rows_y = source_y;
                    let mut rows = vec![0; offset.abs() as usize];
                    rows.extend(&shape.rows);
                    shape.rows = rows;
                }
                for (index, row) in source_rows.into_iter().enumerate() {
                    let shape_index = (index as isize + offset) as usize;
                    shape.rows[shape_index] = shape.rows[shape_index] | row;
                }
            }

            let mut new_shapes = vec![];
            for (index, shape) in shapes.into_iter().enumerate() {
                if !to_delete.contains(&index) {
                    new_shapes.push(shape);
                }
            }
            new_shapes
        } else {
            shapes
        }
    }

    fn apply_expansion(
        y: usize,
        mut shapes: &mut Vec<Shape>,
        shape_id: &mut usize,
        expansions: Vec<u128>,
    ) -> Vec<[usize; 2]> {
        let shapes_before = shapes.len();
        let mut merges = vec![];
        let mut trunk = HashMap::new();
        for shape in 0..expansions.len() {
            if shape >= shapes_before {
                shapes.push(Shape {
                    id: *shape_id,
                    contour: false,
                    rows_y: y,
                    rows: vec![expansions[shape]],
                    active: true,
                });
                *shape_id += 1;
            } else {
                let expansion = expansions[shape];
                if expansion != 0 {
                    match trunk.get(&expansion) {
                        None => {
                            shapes[shape].rows.push(expansion);
                            trunk.insert(expansion, shape);
                        }
                        Some(trunk) => {
                            merges.push([shape, *trunk]);
                        }
                    }
                } else {
                    shapes[shape].active = false;
                }
            }
        }
        merges
    }

    pub fn expand_shapes_by_row(row: u128, shapes: Vec<u128>) -> Vec<u128> {
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

    pub fn merge_shapes_into_buildings(mut shapes: Vec<Shape>) -> Vec<Shape> {
        loop {
            let mut merge = None;
            'collision_detection: for source_index in 1..shapes.len() {
                for destination_index in 1..shapes.len() {
                    if source_index == destination_index {
                        continue;
                    }
                    let source = &shapes[source_index];
                    let destination = &shapes[destination_index];
                    let offset = source.rows_y as isize - destination.rows_y as isize;
                    if offset < 0 || offset >= destination.rows.len() as isize {
                        continue;
                    }
                    let offset = offset as usize;
                    let overlaps = source.rows.len().min(destination.rows.len() - offset);
                    for i in 0..overlaps {
                        if destination.rows[i + offset] & source.rows[i] != 0 {
                            merge = Some([source_index, destination_index]);
                            break 'collision_detection;
                        }
                    }
                }
            }
            if let Some(merge) = merge {
                shapes = Self::merge_shapes(vec![merge], shapes);
            } else {
                break;
            }
        }
        shapes
    }

    pub fn calculate_shapes(
        map: &[[PlatformCell; Platform::SIZE_X]; Platform::SIZE_Y],
    ) -> Vec<Shape> {
        let exterior_shape = Shape {
            id: Shape::EXTERIOR_ID,
            contour: false,
            rows_y: 0,
            rows: vec![u128::MAX],
            active: true,
        };
        let mut unique_id = 1;
        let mut shapes: Vec<Shape> = vec![exterior_shape];
        for y in 1..Platform::SIZE_Y {
            let mut row = 0;
            for x in 0..Platform::SIZE_X {
                if !map[y][x].wall {
                    row = row | 1 << (Platform::SIZE_X - x - 1);
                }
            }
            let shapes_above_row: Vec<u128> = shapes
                .iter()
                .map(|shape| match shape.active {
                    true => *shape.rows.last().unwrap(),
                    false => 0,
                })
                .collect();
            let expansions = Self::expand_shapes_by_row(row, shapes_above_row);
            let merges = Self::apply_expansion(y, &mut shapes, &mut unique_id, expansions);
            shapes = Self::merge_shapes(merges, shapes);
        }
        Self::grow_shapes(&mut shapes);
        let shapes = Self::merge_shapes_into_buildings(shapes);
        shapes
    }
}

pub fn encode_platform_map(
    map: [[PlatformCell; Platform::SIZE_X]; Platform::SIZE_Y],
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
                    }
                })
                .collect(),
        );
    }
    map
}
