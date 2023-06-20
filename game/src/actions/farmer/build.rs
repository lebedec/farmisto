use crate::api::{ActionError, Event};
use crate::building::{Building, BuildingDomain, GridId, Marker, Material, Structure};
use crate::inventory::FunctionsQuery;
use crate::math::TileMath;
use crate::model::{Construction, Farmer, Farmland};
use crate::physics::{Physics, PhysicsDomain, SpaceId};
use crate::{occur, Game};

pub struct Aggregate2D<'p, 'b> {
    physics: Box<dyn FnOnce() -> Vec<Physics> + 'p>,
    building: Box<dyn FnOnce() -> Vec<Building> + 'b>,
}

impl<'p, 'b> Aggregate2D<'p, 'b> {
    pub fn commit(self) -> Event {
        // occur![(self.building)(), (self.physics)(),]
        unimplemented!()
    }
}

impl Game {
    pub(crate) fn add_wall<'b, 'p>(
        building: &'b mut BuildingDomain,
        physics: &'p mut PhysicsDomain,
    ) -> Result<Aggregate2D<'p, 'b>, ActionError> {
        let (_structure, create_wall) = building.create_wall(GridId(0), [0, 0], Material(0))?;
        let create_hole = physics.create_hole(SpaceId(0), [0, 0], 1)?;
        Ok(Aggregate2D {
            physics: Box::new(create_hole),
            building: Box::new(create_wall),
        })
    }

    pub(crate) fn add_wall_facade<'b, 'p>(
        &'static mut self,
    ) -> Result<Aggregate2D<'p, 'b>, ActionError> {
        // Self::add_wall(&mut self.building, &mut self.physics)

        let building = &mut self.building;
        let physics = &mut self.physics;
        let (_structure, create_wall) = building.create_wall(GridId(0), [0, 0], Material(0))?;
        let create_hole = physics.create_hole(SpaceId(0), [0, 0], 1)?;
        Ok(Aggregate2D {
            physics: Box::new(create_hole),
            building: Box::new(create_wall),
        })
    }

    pub fn build2(
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

        let item = self.inventory.get_container_item(construction.container)?;
        let _material_index = item.kind.functions.as_material()?;
        self.ensure_tile_empty(farmland, tile)?;
        let use_items = self.inventory.use_items_from(construction.container)?;

        let aggregate = Self::add_wall(&mut self.building, &mut self.physics)?;
        // let aggregate = self.add_wall_facade()?;

        // let (structure, create_wall) = self
        //     .building
        //     .create_wall(GridId(0), [0, 0], Material(0))
        //     .unwrap();

        let events = occur![
            use_items(),
            aggregate.commit(),
            self.universe.vanish_construction(construction),
        ];
        Ok(events)
    }

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
