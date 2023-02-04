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
    pub marker: bool,
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

impl Grid {
    pub const COLUMNS: usize = 128;
    pub const ROWS: usize = 128;

    pub fn default_map() -> Vec<Vec<Cell>> {
        vec![vec![Cell::default(); Grid::COLUMNS]; Grid::ROWS]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SurveyorId(pub(crate) usize);

pub struct Surveyor {
    id: SurveyorId,
    grid: GridId,
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
    pub grids_sequence: usize,
    pub surveyors: Vec<Surveyor>,
}

impl BuildingDomain {
    pub fn load_grids(&mut self, grids: Vec<Grid>, sequence: usize) {
        self.grids_sequence = sequence;
        self.grids.extend(grids);
    }

    #[inline]
    pub fn get_grid(&self, id: GridId) -> &Grid {
        self.grids.iter().find(|grid| grid.id == id).unwrap()
    }

    pub fn create_grid(&mut self, kind: Shared<GridKind>) -> GridId {
        self.grids_sequence += 1;
        let id = GridId(self.grids_sequence);
        self.grids.push(Grid {
            id,
            kind,
            cells: vec![vec![Cell::default(); Grid::COLUMNS]; Grid::ROWS],
            rooms: vec![],
        });
        id
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

    pub fn create_wall(
        &mut self,
        grid_id: GridId,
        cell: [usize; 2],
        material: Material,
    ) -> Vec<Building> {
        let grid = self.grids.get_mut(grid_id.0).unwrap();
        let [cell_x, cell_y] = cell;
        grid.cells[cell_y][cell_x].wall = true;
        grid.rooms = Grid::calculate_rooms(&grid.cells);
        vec![Building::GridChanged {
            grid: grid_id,
            cells: grid.cells.clone(),
            rooms: grid.rooms.clone(),
        }]
    }
}
