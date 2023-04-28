use crate::api::{ActionError, Event};
use crate::landscaping::Landscaping;
use crate::math::{Array2D, VectorMath};
use crate::model::{
    Assembly, Cementer, Composter, Creature, Crop, Door, Equipment, Farmer, ItemData, PlayerId,
    Rest, Stack, Universe, UniverseSnapshot,
};
use crate::physics::Physics;
use crate::planting::Planting;
use crate::Game;
use log::info;
use std::time::Instant;

impl Game {
    pub fn inspect_player_private_space(
        &self,
        player: PlayerId,
    ) -> Result<Vec<Event>, ActionError> {
        let mut events = vec![];
        let limit_x = 128;
        let limit_y = 128;
        let range = [36, 20];
        let farmer = self.universe.get_player_farmer(player)?;
        let body = self.physics.get_body(farmer.body)?;
        let farmland = self.universe.get_farmland_by_space(body.space)?;
        let [x, y] = body.position.to_tile();
        let [w, h] = range;
        let x = if x >= w / 2 { x - w / 2 } else { 0 };
        let y = if y >= h / 2 { y - h / 2 } else { 0 };
        let w = if x + w < limit_x { x + w } else { limit_x - x };
        let h = if y + h < limit_y { y + h } else { limit_y - y };
        let rect = [x, y, w, h];
        let land = self.landscaping.get_land(farmland.land)?;
        let surface = land.surface.extract_rect(land.kind.width, rect);
        let moisture = land.moisture.extract_rect(land.kind.width, rect);
        let moisture_capacity = land.moisture_capacity.extract_rect(land.kind.width, rect);
        events.push(
            vec![
                Landscaping::SurfaceInspected {
                    land: land.id,
                    rect,
                    surface,
                },
                Landscaping::MoistureCapacityInspected {
                    land: land.id,
                    rect,
                    moisture_capacity,
                },
                Landscaping::MoistureInspected {
                    land: land.id,
                    rect,
                    moisture,
                },
            ]
            .into(),
        );
        let soil = self.planting.get_soil(farmland.soil)?;
        let fertility = soil.fertility.extract_rect(soil.kind.width, rect);
        events.push(
            vec![Planting::SoilFertilityInspected {
                soil: soil.id,
                rect,
                fertility,
            }]
            .into(),
        );
        Ok(events)
    }

    pub fn inspect_crop(&self, entity: Crop) -> Result<Universe, ActionError> {
        let plant = self.planting.get_plant(entity.plant)?;
        let barrier = self.physics.get_barrier(entity.barrier)?;
        let event = Universe::CropAppeared {
            entity,
            impact: plant.impact,
            thirst: plant.thirst,
            hunger: plant.hunger,
            growth: plant.growth,
            health: plant.health,
            fruits: plant.fruits,
            position: barrier.position,
        };
        Ok(event)
    }

    pub fn look_at_creature(&self, entity: Creature) -> Universe {
        let animal = self.raising.get_animal(entity.animal).unwrap();
        let body = self.physics.get_body(entity.body).unwrap();
        Universe::CreatureAppeared {
            entity,
            space: body.space,
            health: animal.health,
            hunger: animal.hunger,
            position: body.position,
        }
    }

    pub fn look_at_assembly(&self, entity: Assembly) -> Universe {
        let placement = self.assembling.get_placement(entity.placement).unwrap();
        Universe::AssemblyAppeared {
            entity,
            rotation: placement.rotation,
            pivot: placement.pivot,
            valid: placement.valid,
        }
    }

    pub fn look_at_door(&self, entity: Door) -> Universe {
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        let placement = self.assembling.get_placement(entity.placement).unwrap();
        Universe::DoorAppeared {
            entity,
            open: !barrier.active,
            rotation: placement.rotation,
            position: barrier.position,
        }
    }

    pub fn look_at_rest(&self, entity: Rest) -> Universe {
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        let placement = self.assembling.get_placement(entity.placement).unwrap();
        Universe::RestAppeared {
            entity,
            rotation: placement.rotation,
            position: barrier.position,
        }
    }

    pub fn inspect_cementer(&self, entity: Cementer) -> Universe {
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        let placement = self.assembling.get_placement(entity.placement).unwrap();
        let device = self.working.get_device(entity.device).unwrap();
        Universe::CementerAppeared {
            entity,
            rotation: placement.rotation,
            position: barrier.position,
            enabled: device.enabled,
            broken: device.broken,
            input: device.input,
            output: device.output,
            progress: device.progress,
        }
    }

    pub fn inspect_composter(&self, entity: Composter) -> Universe {
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        let placement = self.assembling.get_placement(entity.placement).unwrap();
        let device = self.working.get_device(entity.device).unwrap();
        Universe::ComposterInspected {
            entity,
            rotation: placement.rotation,
            position: barrier.position,
            enabled: device.enabled,
            broken: device.broken,
            input: device.input,
            output: device.output,
            progress: device.progress,
        }
    }

    pub fn look_at_stack(&self, stack: Stack) -> Universe {
        let barrier = self.physics.get_barrier(stack.barrier).unwrap();
        Universe::StackAppeared {
            stack,
            position: barrier.position,
        }
    }

    pub fn look_at_equipment(&self, entity: Equipment) -> Universe {
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        Universe::EquipmentAppeared {
            entity,
            position: barrier.position,
        }
    }

    pub fn look_around(&self, snapshot: UniverseSnapshot) -> Vec<Event> {
        let mut stream = vec![];

        for farmland in self.universe.farmlands.iter() {
            if snapshot.whole || snapshot.farmlands.contains(&farmland.id) {
                let soil = self.planting.get_soil(farmland.soil).unwrap();
                let grid = self.building.get_grid(farmland.grid).unwrap();
                let space = self.physics.get_space(farmland.space).unwrap();
                let land = self.landscaping.get_land(farmland.land).unwrap();
                let calendar = self.timing.get_calendar(farmland.calendar).unwrap();
                stream.push(Universe::FarmlandAppeared {
                    farmland: *farmland,
                    // moisture: land.moisture,
                    // moisture_capacity: land.moisture_capacity,
                    cells: grid.cells.clone(),
                    rooms: grid.rooms.clone(),
                    holes: space.holes.clone(),
                    season: calendar.season,
                    season_day: calendar.season_day,
                    times_of_day: calendar.times_of_day,
                });
            }
        }

        for farmer in self.universe.farmers.iter() {
            if snapshot.whole || snapshot.farmers.contains(&farmer.id) {
                let body = self.physics.get_body(farmer.body).unwrap();
                let player = self
                    .players
                    .iter()
                    .find(|player| player.id == farmer.player)
                    .unwrap();
                stream.push(Universe::FarmerAppeared {
                    farmer: *farmer,
                    player: player.name.clone(),
                    position: body.position,
                })
            }
        }

        for stack in &self.universe.stacks {
            stream.push(self.look_at_stack(*stack));
        }

        for construction in &self.universe.constructions {
            stream.push(Universe::ConstructionAppeared {
                id: *construction,
                cell: construction.cell,
            })
        }

        for crop in &self.universe.crops {
            stream.push(self.inspect_crop(*crop).unwrap());
        }

        for creature in &self.universe.creatures {
            stream.push(self.look_at_creature(*creature));
        }

        for equipment in &self.universe.equipments {
            stream.push(self.look_at_equipment(*equipment));
        }

        for assembly in &self.universe.assembly {
            stream.push(self.look_at_assembly(*assembly));
        }

        for door in &self.universe.doors {
            stream.push(self.look_at_door(*door));
        }

        for rest in &self.universe.rests {
            stream.push(self.look_at_rest(*rest));
        }

        for cementer in &self.universe.cementers {
            stream.push(self.inspect_cementer(*cementer));
        }

        for composter in &self.universe.composters {
            stream.push(self.inspect_composter(*composter));
        }

        let mut items_appearance = vec![];
        for container in self.inventory.containers.values() {
            for item in &container.items {
                items_appearance.push(ItemData {
                    id: item.id,
                    kind: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                })
            }
        }
        stream.push(Universe::ItemsAppeared {
            items: items_appearance,
        });

        // client uses barrier hints to simulate physics locally to smooth network lag
        // so, send information about all barriers like it just created
        let barriers_hint = self.physics.barriers[1]
            .iter()
            .map(|barrier| Physics::BarrierCreated {
                id: barrier.id,
                key: barrier.kind.id,
                space: barrier.space,
                position: barrier.position,
                active: barrier.active,
            })
            .collect();

        // let surfaces = self
        //     .landscaping
        //     .lands
        //     .values()
        //     .map(|land| Landscaping::SurfaceUpdate {
        //         land: land.id,
        //         surface: land.surface,
        //     })
        //     .collect();

        vec![
            Event::UniverseStream(stream),
            Event::PhysicsStream(barriers_hint),
            // Event::LandscapingStream(surfaces),
        ]
    }
}