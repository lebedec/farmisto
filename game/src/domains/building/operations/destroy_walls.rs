use crate::building::BuildingError::{CellHasNoWall, CellHasWall};
use crate::building::{Building, BuildingDomain, BuildingError, Grid, GridId};

impl BuildingDomain {
    pub fn destroy_walls<'operation>(
        &'operation mut self,
        grid_id: GridId,
        cells: Vec<[usize; 2]>,
    ) -> Result<impl FnOnce() -> Vec<Building> + 'operation, BuildingError> {
        let grid = self.get_mut_grid(grid_id)?;
        for cell in &cells {
            if !grid.get_cell_mut(*cell).wall {
                return Err(CellHasNoWall { cell: *cell });
            }
        }
        let operation = move || {
            for cell in cells {
                let cell = grid.get_cell_mut(cell);
                cell.wall = false;
                cell.window = false;
                cell.door = false;
            }
            grid.rooms = Grid::calculate_rooms(&grid.cells);
            vec![Building::GridChanged {
                grid: grid_id,
                cells: grid.cells.clone(),
                rooms: grid.rooms.clone(),
            }]
        };
        Ok(operation)
    }
}
