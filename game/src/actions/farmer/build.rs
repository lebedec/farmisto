use crate::api::{ActionError, Event};
use crate::building::{Marker, Material, Structure};
use crate::inventory::FunctionsQuery;
use crate::math::TileMath;
use crate::model::{Construction, Farmer, Farmland};
use crate::{occur, Game};

impl Game {
    pub(crate) fn build(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
        construction: Construction,
    ) -> Result<Vec<Event>, ActionError> {
        let surveyor = self.building.get_surveyor(construction.surveyor)?;
        let stake = surveyor.get_stake(construction.stake)?;
        let tile = stake.cell;
        let destination = stake.cell.position();
        self.ensure_target_reachable(farmer.body, destination)?;
        match stake.marker {
            Marker::Construction(_) => {
                let item = self.inventory.get_container_item(construction.container)?;
                let material_index = item.kind.functions.as_material()?;
                let material = Material(material_index);
                self.ensure_tile_empty(farmland, tile)?;

                let use_items = self.inventory.use_items_from(construction.container)?;
                let (structure, create_wall) =
                    self.building.create_wall(farmland.grid, tile, material)?;
                let size = if material_index == Material::PLANKS {
                    2
                } else {
                    1
                };
                let create_hole = self.physics.create_hole(farmland.space, tile, size)?;

                if structure == Structure::Door {
                    let events = occur![
                        use_items(),
                        create_wall(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                } else {
                    let events = occur![
                        use_items(),
                        create_wall(),
                        create_hole(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                }
            }
            Marker::Reconstruction(_structure) => {
                let grid = self.building.get_grid(construction.grid)?;
                let material = grid.cells[tile[1]][tile[0]].material;
                let (structure, create_wall) =
                    self.building.create_wall(farmland.grid, tile, material)?;
                let size = if material.0 == Material::PLANKS { 2 } else { 1 };
                let create_hole = self.physics.create_hole(farmland.space, tile, size)?;

                if structure == Structure::Door {
                    let events = occur![
                        // use_items(),
                        create_wall(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                } else {
                    let events = occur![
                        // use_items(),
                        create_wall(),
                        create_hole(),
                        self.universe.vanish_construction(construction),
                    ];
                    Ok(events)
                }
            }
            Marker::Deconstruction => {
                let use_items = self.inventory.use_items_from(construction.container)?;
                let destroy_wall = self.building.destroy_walls(farmland.grid, vec![tile])?;
                let destroy_hole = self.physics.destroy_hole(farmland.space, tile)?;

                let events = occur![
                    use_items(),
                    destroy_wall(),
                    destroy_hole(),
                    self.universe.vanish_construction(construction),
                ];
                Ok(events)
            }
        }
    }
}
