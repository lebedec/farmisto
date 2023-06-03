use crate::api::{ActionError, Event};
use crate::inventory::{Inventory, ItemData};
use crate::landscaping::Landscaping;
use crate::math::{Array, ArrayIndex, VectorMath};
use crate::model::{
    Assembly, Cementer, Composter, Corpse, Creature, Crop, Door, Equipment, Farmer, Farmland,
    PlayerId, Rest, Stack, Universe, UniverseSnapshot,
};
use crate::physics::Physics;
use crate::planting::Planting;
use crate::Game;

impl Game {
    pub fn inspect_player_private_space(
        &self,
        player: PlayerId,
    ) -> Result<Vec<Event>, ActionError> {
        let mut events = vec![];
        let limit_x = 128;
        let limit_y = 128;
        let range = [37, 21];
        let farmer = self.universe.get_player_farmer(player)?;
        let body = self.physics.get_body(farmer.body)?;
        let farmland = self.universe.get_farmland_by_space(body.space)?;
        let rect = body.position.to_tile().rect([limit_x, limit_y], range);
        let land = self.landscaping.get_land(farmland.land)?;
        let surface = land.surface.copy(land.kind.width, rect);
        let moisture = land.moisture.copy(land.kind.width, rect);
        let moisture_capacity = land.moisture_capacity.copy(land.kind.width, rect);
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
        let fertility = soil.fertility.copy(soil.kind.width, rect);
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

    pub fn inspect_creature(&self, entity: Creature) -> Result<Universe, ActionError> {
        let animal = self.raising.get_animal(entity.animal)?;
        let body = self.physics.get_body(entity.body)?;
        let farmland = self.universe.get_farmland_by_space(body.space)?;
        let event = Universe::CreatureAppeared {
            entity,
            farmland,
            health: animal.health,
            hunger: animal.hunger,
            age: animal.age,
            weight: animal.weight,
            position: body.position,
            behaviour: animal.behaviour,
        };
        Ok(event)
    }

    pub fn inspect_corpse(&self, entity: Corpse) -> Result<Universe, ActionError> {
        let barrier = self.physics.get_barrier(entity.barrier)?;
        let event = Universe::CorpseAppeared {
            entity,
            position: barrier.position,
        };
        Ok(event)
    }

    pub fn inspect_assembly(&self, entity: Assembly) -> Universe {
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

    pub fn inspect_stack(&self, stack: Stack) -> Universe {
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

    pub fn inspect_farmer(&self, farmer: Farmer) -> Result<Universe, ActionError> {
        let body = self.physics.get_body(farmer.body)?;
        let player = self
            .players
            .iter()
            .find(|player| player.id == farmer.player)
            .unwrap();
        Ok(Universe::FarmerAppeared {
            farmer,
            player: player.name.clone(),
            position: body.position,
        })
    }

    pub fn inspect_farmland(&self, farmland: Farmland) -> Result<Universe, ActionError> {
        let _soil = self.planting.get_soil(farmland.soil).unwrap();
        let grid = self.building.get_grid(farmland.grid).unwrap();
        let space = self.physics.get_space(farmland.space).unwrap();
        let _land = self.landscaping.get_land(farmland.land).unwrap();
        let calendar = self.timing.get_calendar(farmland.calendar).unwrap();
        Ok(Universe::FarmlandAppeared {
            farmland,
            cells: grid.cells.clone(),
            rooms: grid.rooms.clone(),
            holes: space.holes.clone(),
            season: calendar.season,
            season_day: calendar.season_day,
            times_of_day: calendar.times_of_day,
        })
    }

    pub fn look_around(&self, snapshot: UniverseSnapshot) -> Vec<Event> {
        let mut stream = vec![];

        for farmland in self.universe.farmlands.iter() {
            if snapshot.whole || snapshot.farmlands.contains(&farmland.id) {
                stream.push(self.inspect_farmland(*farmland).unwrap());
            }
        }

        for farmer in self.universe.farmers.iter() {
            stream.push(self.inspect_farmer(*farmer).unwrap());
        }

        for stack in &self.universe.stacks {
            stream.push(self.inspect_stack(*stack));
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
            stream.push(self.inspect_creature(*creature).unwrap());
        }

        for corpse in &self.universe.corpses {
            stream.push(self.inspect_corpse(*corpse).unwrap());
        }

        for equipment in &self.universe.equipments {
            stream.push(self.look_at_equipment(*equipment));
        }

        for assembly in &self.universe.assembly {
            stream.push(self.inspect_assembly(*assembly));
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

        let mut items = vec![];
        for container in self.inventory.containers.values() {
            for item in &container.items {
                items.push(ItemData {
                    id: item.id,
                    key: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                })
            }
        }
        // TODO: scoped info about items
        let all_game_items = vec![Inventory::ItemsAdded { items }];

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

        vec![
            Event::UniverseStream(stream),
            Event::PhysicsStream(barriers_hint),
            Event::InventoryStream(all_game_items),
        ]
    }
}
