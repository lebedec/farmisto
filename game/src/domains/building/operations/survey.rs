use crate::building::{Building, BuildingDomain, BuildingError, Marker, SurveyorId};

impl BuildingDomain {
    pub fn survey<'operation>(
        &'operation mut self,
        _surveyor: SurveyorId,
        cell: [usize; 2],
        marker: Marker,
    ) -> Result<impl FnOnce() -> Vec<Building> + 'operation, BuildingError> {
        let grid = &mut self.grids[0];
        let [column, row] = cell;
        if grid.cells[row][column].wall {
            return Err(BuildingError::CellHasWall { cell });
        }
        let operation = move || {
            grid.cells[row][column].wall = true;
            grid.cells[row][column].marker = Some(marker);
            vec![Building::GridChanged {
                grid: grid.id,
                cells: grid.cells.clone(),
                rooms: grid.rooms.clone(),
            }]
        };
        Ok(operation)
    }
}
