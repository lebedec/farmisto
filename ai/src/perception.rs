use game::api::Event;
use game::collections::Shared;
use game::inventory::{ContainerId, FunctionsQuery, Inventory, ItemId, ItemKey, ItemKind};
use game::math::{Position, VectorMath};
use game::model::{Creature, Crop, Knowledge, Universe};
use game::physics::Physics;
use game::raising::Raising;
use log::error;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::decision_making::Thinking;
use crate::fauna::CreatureAgent;
use crate::Nature;

pub struct CropView {
    pub entity: Crop,
    pub growth: f32,
    pub position: [f32; 2],
}

pub struct FarmerView {}

pub struct CreatureView {
    pub _entity: Creature,
}

#[derive(Default, Clone, Copy)]
pub struct TileView {
    pub has_hole: bool,
    pub has_barrier: bool,
}

pub struct InvaserView {
    _threat: f32,
}

pub struct ContainerView {
    pub id: ContainerId,
    pub position: Position,
}

#[derive(Clone)]
pub struct FoodView {
    pub item: ItemId,
    pub container: ContainerId,
    pub quantity: u8,
    pub max_quantity: u8,
    pub position: Position,
}

impl Nature {
    pub fn perceive_universe(&mut self, event: Universe, known: &Knowledge) {
        match event {
            Universe::ActivityChanged { .. } => {}
            Universe::TreeAppeared { .. } => {}
            Universe::TreeVanished(_) => {}
            Universe::FarmlandAppeared {
                farmland, holes, ..
            } => {
                let mut tiles = vec![vec![TileView::default(); 128]; 128];
                for y in 0..holes.len() {
                    for x in 0..holes.len() {
                        tiles[y][x].has_hole = holes[y][x] == 1;
                    }
                }
                self.tiles.insert(farmland.space, tiles);
            }
            Universe::FarmlandVanished(_) => {}
            Universe::FarmerAppeared { .. } => {}
            Universe::FarmerVanished(_) => {}
            Universe::StackAppeared { stack, position } => {
                let view = ContainerView {
                    id: stack.container,
                    position,
                };
                self.containers.insert(stack.container, view);
            }
            Universe::StackVanished(stack) => {
                self.containers.remove(&stack.container);
            }
            Universe::CropAppeared {
                entity,
                growth,
                position,
                ..
            } => self.crops.push(CropView {
                entity,
                growth,
                position,
            }),
            Universe::CropVanished(_) => {}
            Universe::ConstructionAppeared { .. } => {}
            Universe::ConstructionVanished { .. } => {}
            Universe::EquipmentAppeared { .. } => {}
            Universe::EquipmentVanished(_) => {}
            Universe::CreatureAppeared {
                entity,
                health,
                position,
                hunger,
                space,
            } => {
                self.creatures.push(CreatureView { _entity: entity });
                self.creature_agents.push(CreatureAgent {
                    entity: entity,
                    last_action: Instant::now(),
                    space,
                    hunger,
                    health,
                    thinking: Thinking::default(),
                    position,
                    radius: 5.0,
                    thirst: 0.0,
                })
            }
            Universe::CreatureEats { .. } => {}
            Universe::CreatureVanished(creature) => {
                match self
                    .creature_agents
                    .iter()
                    .position(|agent| agent.entity == creature)
                {
                    None => {
                        error!("Unable to remove creature agent {creature:?}, not found");
                    }
                    Some(index) => {
                        self.creatures.remove(index);
                    }
                }
                match self
                    .creatures
                    .iter()
                    .position(|agent| agent._entity == creature)
                {
                    None => {
                        error!("Unable to remove creature view {creature:?}, not found");
                    }
                    Some(index) => {
                        self.creatures.remove(index);
                    }
                }
            }
            Universe::AssemblyAppeared { .. } => {}
            Universe::AssemblyUpdated { .. } => {}
            Universe::AssemblyVanished(_) => {}
            Universe::DoorAppeared { .. } => {}
            Universe::DoorVanished(_) => {}
            Universe::DoorChanged { .. } => {}
            Universe::CementerAppeared { .. } => {}
            Universe::CementerVanished(_) => {}
            Universe::RestAppeared { .. } => {}
            Universe::RestVanished(_) => {}
            Universe::ComposterInspected { .. } => {}
            Universe::ComposterVanished(_) => {}
            Universe::CorpseAppeared { .. } => {}
            Universe::CorpseVanished(_) => {}
        }
    }

    pub fn perceive_physics(&mut self, event: Physics) {
        match event {
            Physics::BodyPositionChanged { id, position, .. } => {
                for agent in self.creature_agents.iter_mut() {
                    if agent.entity.body == id {
                        agent.position = position;
                        break;
                    }
                }
            }
            Physics::BarrierCreated {
                space, position, ..
            } => {
                let tiles = self.tiles.get_mut(&space).expect("tiles");
                let [x, y] = position.to_tile();
                // TODO: barrier bounds
                tiles[y][x].has_barrier = true;
            }
            Physics::BarrierChanged { id, space, active } => {
                // let tiles = self.tiles.get_mut(&space).expect("tiles");
                // let [x, y] = position.to_tile();
                // tiles[y][x].has_barrier = active;
            }
            Physics::BarrierDestroyed {
                position, space, ..
            } => {
                let tiles = self.tiles.get_mut(&space).expect("tiles");
                let [x, y] = position.to_tile();
                // TODO: multiple barriers on same tile
                tiles[y][x].has_barrier = false;
            }
            Physics::SpaceUpdated { id, holes } => {
                let tiles = self.tiles.get_mut(&id).expect("tiles");
                for y in 0..holes.len() {
                    for x in 0..holes.len() {
                        tiles[y][x].has_hole = holes[y][x] == 1;
                    }
                }
            }
        }
    }

    pub fn perceive_raising(&mut self, event: Raising) {
        match event {
            Raising::AnimalChanged { id, hunger, thirst } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.hunger = hunger;
                        creature.thirst = thirst;
                        break;
                    }
                }
            }
            Raising::AnimalDamaged { id, health } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.health = health;
                        break;
                    }
                }
            }
            Raising::LeadershipChanged { .. } => {}
            Raising::HerdsmanChanged { .. } => {}
        }
    }

    pub fn perceive_inventory(&mut self, event: Inventory, known: &Knowledge) {
        match event {
            Inventory::ContainerCreated { .. } => {}
            Inventory::ContainerDestroyed { .. } => {}
            Inventory::ItemsAdded { items } => {
                for item in items {
                    let kind = known.items.get(item.key).unwrap();
                    let container = self.containers.get(&item.container).unwrap();
                    if kind.functions.as_food().is_ok() {
                        self.foods.insert(
                            item.id,
                            FoodView {
                                item: item.id,
                                container: item.container,
                                quantity: item.quantity,
                                max_quantity: kind.max_quantity,
                                position: container.position,
                            },
                        );
                    }
                }
            }
            Inventory::ItemRemoved { item, .. } => {
                self.foods.remove(&item);
            }
            Inventory::ItemQuantityChanged { quantity, id, .. } => {
                self.foods
                    .entry(id)
                    .and_modify(|food| food.quantity = quantity);
            }
        }
    }

    pub fn perceive(&mut self, events: Vec<Event>, knowledge: &Knowledge) {
        for event in events {
            match event {
                Event::UniverseStream(events) => {
                    for event in events {
                        self.perceive_universe(event, knowledge);
                    }
                }
                Event::PhysicsStream(events) => {
                    for event in events {
                        self.perceive_physics(event)
                    }
                }
                Event::BuildingStream(_) => {}
                Event::InventoryStream(events) => {
                    for event in events {
                        self.perceive_inventory(event, knowledge);
                    }
                }
                Event::PlantingStream(_) => {}
                Event::RaisingStream(events) => {
                    for event in events {
                        self.perceive_raising(event)
                    }
                }
                Event::AssemblingStream(_) => {}
                Event::WorkingStream(_) => {}
                Event::LandscapingStream(_) => {}
                Event::TimingStream(_) => {}
            }
        }
    }
}
