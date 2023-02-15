use crate::building::BuildingError::{CellHasNoMarkers, CellHasWall};
use crate::building::{Building, BuildingDomain, BuildingError, Grid, GridId, Marker, Material};

impl BuildingDomain {
    pub fn create_wall<'operation>(
        &'operation mut self,
        grid_id: GridId,
        cell: [usize; 2],
        material: Material,
    ) -> Result<(Marker, impl FnOnce() -> Vec<Building> + 'operation), BuildingError> {
        let grid = self.get_mut_grid(grid_id)?;
        let [cell_x, cell_y] = cell;

        let marker = grid.cells[cell_y][cell_x]
            .marker
            .ok_or(CellHasNoMarkers { cell })?;

        // if grid.cells[cell_y][cell_x].wall {
        //     return Err(CellHasWall { cell });
        // }

        let operation = move || {
            grid.cells[cell_y][cell_x].marker = None;
            match marker {
                Marker::Wall => {
                    grid.cells[cell_y][cell_x].wall = true;
                }
                Marker::Window => {
                    grid.cells[cell_y][cell_x].wall = true;
                    grid.cells[cell_y][cell_x].window = true;
                }
                Marker::Door => {
                    grid.cells[cell_y][cell_x].wall = true;
                    grid.cells[cell_y][cell_x].door = true;
                }
            }
            grid.rooms = Grid::calculate_rooms(&grid.cells);
            vec![Building::GridChanged {
                grid: grid_id,
                cells: grid.cells.clone(),
                rooms: grid.rooms.clone(),
            }]
        };

        Ok((marker, operation))
    }
}
