use crate::building::{Building, BuildingDomain, BuildingError, Grid, GridId, GridKind};
use crate::collections::Shared;

impl BuildingDomain {
    pub fn create_grid(
        &mut self,
        kind: &Shared<GridKind>,
    ) -> Result<(GridId, impl FnOnce() -> Vec<Building> + '_), BuildingError> {
        let id = GridId(self.grids_sequence + 1);
        let cells = Grid::default_map();
        let rooms = Grid::calculate_rooms(&cells);
        let grid = Grid {
            id,
            kind: kind.clone(),
            cells,
            rooms,
        };
        let command = move || {
            let events = vec![];
            self.grids_sequence += 1;
            self.grids.push(grid);
            events
        };
        Ok((id, command))
    }
}
