use crate::api::Event;
use crate::model::{
    Assembly, Cementer, Creature, Crop, Door, Equipment, ItemData, Stack, Universe,
    UniverseSnapshot,
};
use crate::physics::Physics;
use crate::Game;

impl Game {
    pub fn look_at_crop(&self, entity: Crop) -> Universe {
        let plant = self.planting.get_plant(entity.plant).unwrap();
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        Universe::CropAppeared {
            entity,
            impact: plant.impact,
            thirst: plant.thirst,
            hunger: plant.hunger,
            growth: plant.growth,
            health: plant.health,
            fruits: plant.fruits,
            position: barrier.position,
        }
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

    pub fn look_at_cementer(&self, entity: Cementer) -> Universe {
        let barrier = self.physics.get_barrier(entity.barrier).unwrap();
        let placement = self.assembling.get_placement(entity.placement).unwrap();
        let device = self.working.get_device(entity.device).unwrap();
        Universe::CementerAppeared {
            entity,
            rotation: placement.rotation,
            position: barrier.position,
            mode: device.mode,
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
                let land = self.planting.get_soil(farmland.soil).unwrap();
                let grid = self.building.get_grid(farmland.grid).unwrap();
                let space = self.physics.get_space(farmland.space).unwrap();
                stream.push(Universe::FarmlandAppeared {
                    farmland: *farmland,
                    map: land.map.clone(),
                    cells: grid.cells.clone(),
                    rooms: grid.rooms.clone(),
                    holes: space.holes.clone(),
                })
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
            stream.push(self.look_at_crop(*crop));
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

        vec![
            Event::UniverseStream(stream),
            Event::PhysicsStream(barriers_hint),
        ]
    }
}
