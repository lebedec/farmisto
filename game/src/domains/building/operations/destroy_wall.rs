use crate::building::BuildingError::{CellHasNoWall, CellHasWall};
use crate::building::{Building, BuildingDomain, BuildingError, Grid, GridId};

impl BuildingDomain {
    pub fn destroy_wall<'operation>(
        &'operation mut self,
        grid_id: GridId,
        cell: [usize; 2],
    ) -> Result<impl FnOnce() -> Vec<Building> + 'operation, BuildingError> {
        let grid = self.get_mut_grid(grid_id)?;
        let [cell_x, cell_y] = cell;

        if !grid.cells[cell_y][cell_x].wall {
            return Err(CellHasNoWall { cell });
        }

        let operation = move || {
            grid.cells[cell_y][cell_x].marker = None;
            grid.cells[cell_y][cell_x].wall = false;
            grid.cells[cell_y][cell_x].window = false;
            grid.cells[cell_y][cell_x].door = false;
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
