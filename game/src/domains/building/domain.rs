use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

use crate::collections::Shared;
use crate::math::Rect;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Debug,
    Serialize,
    Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
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

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridId(pub usize);

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: usize,
    pub contour: bool,
    pub area_y: usize,
    pub area: Vec<u128>,
    pub aabb: Rect,
    pub active: bool,
    pub material: Material,
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

    #[inline]
    pub fn get_cell(&self, cell: [usize; 2]) -> Result<&Cell, BuildingError> {
        let [x, y] = cell;
        if y >= self.cells.len() || x >= self.cells[0].len() {
            return Err(BuildingError::CellOutOfBounds { cell });
        }
        Ok(&self.cells[cell[1]][cell[0]])
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Structure {
    Wall,
    Door,
    Window,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Marker {
    Construction(Structure),
    Reconstruction(Structure),
    Deconstruction,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Stake {
    pub id: usize,
    pub marker: Marker,
    pub cell: [usize; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurveyorKey(pub usize);

pub struct SurveyorKind {
    pub id: SurveyorKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SurveyorId(pub usize);

pub struct Surveyor {
    pub id: SurveyorId,
    pub grid: GridId,
    pub stake_id: usize,
    pub surveying: Vec<Stake>,
    pub kind: Shared<SurveyorKind>,
    pub mode: u8,
}

impl Surveyor {
    pub const MODE_WALL: u8 = 0;

    pub const MODE_DOOR: u8 = 1;

    pub const MODE_WINDOW: u8 = 2;

    pub const MODES: [u8; 3] = [Self::MODE_WALL, Self::MODE_DOOR, Self::MODE_WINDOW];
}

#[derive(Serialize, Deserialize)]
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
    SurveyorModeChanged {
        id: SurveyorId,
        mode: u8,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildingError {
    GridNotFound { id: GridId },
    CellOutOfBounds { cell: [usize; 2] },
    CellHasWall { cell: [usize; 2] },
    CellHasNoWall { cell: [usize; 2] },
    CellHasNoMarkers { cell: [usize; 2] },
    SurveyorNotFound { id: SurveyorId },
    SurveyorMarkerNotFound,
    StakeNotFound { id: usize },
    ConstructStakeMarkedForDeconstruction,
}

#[derive(Default)]
pub struct BuildingDomain {
    pub grids: Vec<Grid>,
    pub grids_sequence: usize,
    pub surveyors: Vec<Surveyor>,
    pub surveyors_sequence: usize,
}
