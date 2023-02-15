use crate::collections::Shared;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Material(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub enum Marker {
    Wall,
    Window,
    Door,
}

#[derive(Clone, Copy, Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Cell {
    pub wall: bool,
    pub inner: bool,
    pub door: bool,
    pub window: bool,
    pub marker: Option<Marker>,
    pub material: Material,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridKey(pub usize);

pub struct GridKind {
    pub id: GridKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct GridId(pub usize);

#[derive(Default, Clone, bincode::Encode, bincode::Decode)]
pub struct Room {
    pub id: usize,
    pub contour: bool,
    pub rows_y: usize,
    pub rows: Vec<u128>,
    pub active: bool,
}

impl Debug for Room {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Room")
            .field("id", &self.id)
            .field("contour", &self.contour)
            .field("rows_y", &self.rows_y)
            .field("active", &self.active)
            .finish()
    }
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

impl Debug for Building {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Building::GridChanged { grid, .. } => {
                f.debug_struct("GridChanged").field("grid", grid).finish()
            }
            other => Debug::fmt(other, f),
        }
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum BuildingError {
    GridNotFound { id: GridId },
    CellHasWall { cell: [usize; 2] },
    CellHasNoWall { cell: [usize; 2] },
    CellHasNoMarkers { cell: [usize; 2] },
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
    pub fn get_grid(&self, id: GridId) -> Result<&Grid, BuildingError> {
        self.grids
            .iter()
            .find(|grid| grid.id == id)
            .ok_or(BuildingError::GridNotFound { id })
    }

    #[inline]
    pub fn get_mut_grid(&mut self, id: GridId) -> Result<&mut Grid, BuildingError> {
        self.grids
            .iter_mut()
            .find(|grid| grid.id == id)
            .ok_or(BuildingError::GridNotFound { id })
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

    #[inline]
    pub fn get_surveyor(&self, id: SurveyorId) -> &Surveyor {
        &self.surveyors[id.0]
    }
}
