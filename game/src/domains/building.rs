use crate::collections::Shared;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridKey(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Material(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct GridIndex(pub usize, pub usize);

#[derive(Clone, Copy, Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Cell {
    pub wall: bool,
    pub inner: bool,
    pub door: bool,
    pub window: bool,
    pub material: Material,
}

pub struct GridKind {
    pub id: GridKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct GridId(pub usize);

#[derive(Debug, Default, Clone, bincode::Encode, bincode::Decode)]
pub struct Room {
    pub id: usize,
    pub contour: bool,
    pub rows_y: usize,
    pub rows: Vec<u128>,
    pub active: bool,
}

impl Room {
    pub const EXTERIOR_ID: usize = 0;
}

pub struct Grid {
    pub id: GridId,
    pub kind: Shared<GridKind>,
    pub cells: Vec<Vec<Cell>>,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SurveyorId(usize);

pub struct Surveyor {
    id: SurveyorId,
    grid: GridId,
}

pub struct Surveying<'action> {
    grid: &'action mut Grid,
    pub cell: GridIndex,
}

impl<'action> Surveying<'action> {
    pub fn complete(self) -> Vec<Building> {
        let (column, row) = (self.cell.0, self.cell.1);
        self.grid.cells[column][row].window = true;
        vec![]
    }
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum Building {
    GridChanged {
        grid: GridId,
        cells: Vec<Vec<Cell>>,
        rooms: Vec<Room>,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum BuildingError {
    Occupied { cell: [usize; 2] },
}

#[derive(Default)]
pub struct BuildingDomain {
    pub known_grids: HashMap<GridKey, Shared<GridKind>>,
    pub grids: Vec<Grid>,
    pub surveyors: Vec<Surveyor>,
}

impl BuildingDomain {
    #[inline]
    pub fn get_grid(&self, id: GridId) -> &Grid {
        self.grids.iter().find(|grid| grid.id == id).unwrap()
    }

    pub fn create_grid(&mut self, id: GridId, kind: Shared<GridKind>) {
        self.grids.push(Grid {
            id,
            kind,
            cells: vec![vec![Cell::default(); Grid::COLUMNS]; Grid::ROWS],
            rooms: vec![],
        })
    }

    #[inline]
    pub fn create_surveyor(&self, grid: GridId) -> Result<Surveyor, BuildingError> {
        Ok(Surveyor {
            id: SurveyorId(self.surveyors.len()),
            grid,
        })
    }

    // #[inline]
    // pub fn complete_surveyor_creation(&mut self, surveyor: Surveyor) {
    //     self.surveyors[surveyor.id.0] = surveyor
    // }

    #[inline]
    pub fn get_surveyor(&self, id: SurveyorId) -> &Surveyor {
        &self.surveyors[id.0]
    }

    pub fn survey(
        &mut self,
        surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Result<Surveying, BuildingError> {
        let surveyor = self.get_surveyor(surveyor).grid.0;
        let grid = &mut self.grids[surveyor];
        let [column, row] = cell;
        if grid.cells[row][column].wall {
            return Err(BuildingError::Occupied { cell });
        }
        Ok(Surveying {
            grid,
            cell: GridIndex(column, row),
        })
    }

    pub fn create_wall(
        &mut self,
        grid_id: GridId,
        cell: [usize; 2],
        material: Material,
    ) -> Vec<Building> {
        let grid = self.grids.get_mut(grid_id.0).unwrap();
        let [cell_x, cell_y] = cell;
        grid.cells[cell_y][cell_x].wall = true;
        grid.rooms = Grid::calculate_shapes(&grid.cells);
        vec![Building::GridChanged {
            grid: grid_id,
            cells: grid.cells.clone(),
            rooms: grid.rooms.clone(),
        }]
    }
}

impl Grid {
    pub const COLUMNS: usize = 128;
    pub const ROWS: usize = 128;

    pub fn default_map() -> Vec<Vec<Cell>> {
        vec![vec![Cell::default(); Grid::COLUMNS]; Grid::ROWS]
    }

    fn grow_shapes(shapes: &mut Vec<Room>) {
        for shape in shapes {
            if shape.id == Room::EXTERIOR_ID {
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

    fn merge_shapes(merges: Vec<[usize; 2]>, mut shapes: Vec<Room>) -> Vec<Room> {
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
        mut shapes: &mut Vec<Room>,
        shape_id: &mut usize,
        expansions: Vec<u128>,
    ) -> Vec<[usize; 2]> {
        let shapes_before = shapes.len();
        let mut merges = vec![];
        let mut trunk = HashMap::new();
        for shape in 0..expansions.len() {
            if shape >= shapes_before {
                shapes.push(Room {
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

    pub fn merge_shapes_into_buildings(mut shapes: Vec<Room>) -> Vec<Room> {
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

    pub fn calculate_shapes(map: &Vec<Vec<Cell>>) -> Vec<Room> {
        // TODO: array on stack increases speed to ~2 times!
        // let mut map = [[Cell::default(); Grid::COLUMNS]; Grid::ROWS];
        // for y in 0..Grid::ROWS {
        //     for x in 0..Grid::COLUMNS {
        //         map[y][x] = input_map[y][x];
        //     }
        // }

        let exterior_shape = Room {
            id: Room::EXTERIOR_ID,
            contour: false,
            rows_y: 0,
            rows: vec![u128::MAX],
            active: true,
        };
        let mut unique_id = 1;
        let mut shapes: Vec<Room> = vec![exterior_shape];
        for y in 1..Grid::ROWS {
            let mut row = 0;
            for x in 0..Grid::COLUMNS {
                if !map[y][x].wall {
                    row = row | 1 << (Grid::COLUMNS - x - 1);
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
