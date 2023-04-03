use std::fmt::{Debug, Formatter};

use crate::collections::Shared;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Material(pub u8);

impl Material {
    pub const UNKNOWN: u8 = 0;

    pub const METAL: u8 = 10;

    pub const MESH: u8 = 15;

    pub const CONCRETE: u8 = 20;

    pub const WOOD: u8 = 30;

    pub const PLANKS: u8 = 35;

    pub const GLASS: u8 = 40;

    pub const TARPAULIN: u8 = 50;

    pub fn index(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Cell {
    pub wall: bool,
    pub door: bool,
    pub window: bool,
    pub material: Material,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub area_y: usize,
    pub area: Vec<u128>,
    pub active: bool,
}

impl Debug for Room {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Room")
            .field("id", &self.id)
            .field("contour", &self.contour)
            .field("area_y", &self.area_y)
            .field("active", &self.active)
            .finish()
    }
}

impl Room {
    pub const EXTERIOR_ID: usize = 0;

    pub fn contains(&self, point: [usize; 2]) -> bool {
        let [x, y] = point;
        let x_bit = 1 << (128 - x - 1);
        if y >= self.area_y && y < self.area_y + self.area.len() {
            let row = self.area[y - self.area_y];
            if row & x_bit != 0 {
                return true;
            }
        }
        false
    }
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

    #[inline]
    pub fn get_cell_mut(&mut self, cell: [usize; 2]) -> &mut Cell {
        &mut self.cells[cell[1]][cell[0]]
    }
}

#[derive(
    Clone, Copy, Debug, bincode::Encode, serde::Deserialize, bincode::Decode, PartialEq, Eq, Hash,
)]
pub enum Structure {
    Wall,
    Window,
    Door,
    Fence,
}

#[derive(
    Clone, Copy, Debug, bincode::Encode, serde::Deserialize, bincode::Decode, PartialEq, Eq, Hash,
)]
pub enum Marker {
    Construction(Structure),
    Reconstruction(Structure),
    Deconstruction,
}

#[derive(Clone, Copy, Debug, bincode::Encode, bincode::Decode)]
pub struct Stake {
    pub marker: Marker,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurveyorKey(pub usize);

pub struct SurveyorKind {
    pub id: SurveyorKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SurveyorId(pub usize);

pub struct Surveyor {
    pub id: SurveyorId,
    pub grid: GridId,
    pub surveying: Vec<Stake>,
    pub kind: Shared<SurveyorKind>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum Building {
    GridChanged {
        grid: GridId,
        cells: Vec<Vec<Cell>>,
        rooms: Vec<Room>,
    },
    SurveyorCreated {
        id: SurveyorId,
        grid: GridId,
    },
    SurveyorDestroyed {
        id: SurveyorId,
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
    SurveyorNotFound { id: SurveyorId },
    SurveyorMarkerNotFound,
    ConstructStakeMarkedForDeconstruction,
}

#[derive(Default)]
pub struct BuildingDomain {
    pub grids: Vec<Grid>,
    pub grids_sequence: usize,
    pub surveyors: Vec<Surveyor>,
    pub surveyors_sequence: usize,
}

impl BuildingDomain {
    pub fn load_grids(&mut self, grids: Vec<Grid>, sequence: usize) {
        self.grids_sequence = sequence;
        self.grids.extend(grids);
    }

    pub fn load_surveyors(&mut self, surveyors: Vec<Surveyor>, sequence: usize) {
        self.surveyors_sequence = sequence;
        self.surveyors.extend(surveyors);
    }

    #[inline]
    pub fn get_grid(&self, id: GridId) -> Result<&Grid, BuildingError> {
        self.grids
            .iter()
            .find(|grid| grid.id == id)
            .ok_or(BuildingError::GridNotFound { id })
    }

    pub fn find_surveyor_mut(
        &mut self,
        grid: GridId,
        cell: [usize; 2],
    ) -> Result<&mut Surveyor, BuildingError> {
        self.surveyors
            .iter_mut()
            .find(|surveyor| {
                surveyor.grid == grid && surveyor.surveying.iter().any(|stake| stake.cell == cell)
            })
            .ok_or(BuildingError::SurveyorMarkerNotFound)
    }

    pub fn index_surveyor2(
        &mut self,
        grid: GridId,
        cell: [usize; 2],
    ) -> Result<usize, BuildingError> {
        self.surveyors
            .iter_mut()
            .position(|surveyor| {
                surveyor.grid == grid && surveyor.surveying.iter().any(|stake| stake.cell == cell)
            })
            .ok_or(BuildingError::SurveyorMarkerNotFound)
    }

    #[inline]
    pub fn index_grid(&mut self, id: GridId) -> Result<usize, BuildingError> {
        self.grids
            .iter_mut()
            .position(|grid| grid.id == id)
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
    pub fn get_surveyor(&self, id: SurveyorId) -> Result<&Surveyor, BuildingError> {
        self.surveyors
            .iter()
            .find(|surveyor| surveyor.id == id)
            .ok_or(BuildingError::SurveyorNotFound { id })
    }

    #[inline]
    pub fn get_surveyor_mut(&mut self, id: SurveyorId) -> Result<&mut Surveyor, BuildingError> {
        self.surveyors
            .iter_mut()
            .find(|surveyor| surveyor.id == id)
            .ok_or(BuildingError::SurveyorNotFound { id })
    }

    #[inline]
    pub fn index_surveyor(&self, id: SurveyorId) -> Result<usize, BuildingError> {
        self.surveyors
            .iter()
            .position(|surveyor| surveyor.id == id)
            .ok_or(BuildingError::SurveyorNotFound { id })
    }
}
