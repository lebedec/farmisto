use crate::building::{
    Building, BuildingDomain, BuildingError, Grid, GridId, Marker, Material, Structure,
};
use crate::building::BuildingError::ConstructStakeMarkedForDeconstruction;

impl BuildingDomain {
    pub fn create_wall(
        & mut self,
        grid_id: GridId,
        cell: [usize; 2],
        material: Material,
    ) -> Result<(Structure, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        // TODO: rework find_surveyor_mut
        let grid = self.index_grid(grid_id)?;
        let surveyor = self.find_surveyor_mut(grid_id, cell)?; // 1
        let stake = surveyor
            .surveying
            .iter()
            .position(|stake| stake.cell == cell)
            .unwrap();
        let structure = match surveyor.surveying[stake].marker {
            Marker::Construction(structure) => structure,
            Marker::Reconstruction(structure) => structure,
            Marker::Deconstruction => return Err(ConstructStakeMarkedForDeconstruction),
        };
        let operation = move || {
            self.find_surveyor_mut(grid_id, cell) // 2
                .unwrap()
                .surveying
                .remove(stake);
            let [column, row] = cell;
            let cell = &mut self.grids[grid].cells[row][column];
            cell.material = material;
            match structure {
                Structure::Wall => {
                    cell.wall = true;
                }
                Structure::Window => {
                    cell.wall = true;
                    cell.window = true;
                }
                Structure::Door => {
                    cell.wall = true;
                    cell.door = true;
                }
                Structure::Fence => {
                    cell.wall = true;
                }
            }
            let grid = &mut self.grids[grid];
            grid.rooms = Grid::calculate_rooms(&grid.cells);
            vec![Building::GridChanged {
                grid: grid.id,
                cells: grid.cells.clone(),
                rooms: grid.rooms.clone(),
            }]
        };
        Ok((structure, operation))
    }
}
