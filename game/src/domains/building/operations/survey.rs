use crate::building::{Building, BuildingDomain, BuildingError, Grid, GridIndex, SurveyorId};

pub struct Surveying<'operation> {
    grid: &'operation mut Grid,
    pub cell: GridIndex,
}

impl<'operation> Surveying<'operation> {
    pub fn complete(self) -> Vec<Building> {
        let (row, column) = (self.cell.0, self.cell.1);
        let grid = self.grid;
        grid.cells[column][row].wall = true;
        grid.cells[column][row].marker = true;
        vec![Building::GridChanged {
            grid: grid.id,
            cells: grid.cells.clone(),
            rooms: grid.rooms.clone(),
        }]
    }
}

impl BuildingDomain {
    pub fn survey(
        &mut self,
        _surveyor: SurveyorId,
        cell: [usize; 2],
    ) -> Result<Surveying, BuildingError> {
        // let surveyor = self.get_surveyor(surveyor).grid.0;
        let grid = &mut self.grids[0];
        let [column, row] = cell;
        if grid.cells[row][column].wall {
            return Err(BuildingError::Occupied { cell });
        }
        Ok(Surveying {
            grid,
            cell: GridIndex(column, row),
        })
    }
}
