use std::collections::HashMap;
use std::sync::RwLock;

use log::error;

use game::api::Event;
use game::collections::Shared;
use game::inventory::{ContainerId, FunctionsQuery, Inventory, ItemId, ItemKey, ItemKind};
use game::math::{ArrayIndex, Position, VectorMath};
use game::model::{Creature, Crop, Farmer, Knowledge, Purpose, Stack, Universe};
use game::physics::{BarrierId, BarrierKey, BarrierKind, Physics};
use game::raising::{Raising, TetherId};
use game::timing::Timing;

use crate::decision_making::Thinking;
use crate::fauna::{CreatureAgent, Targeting};
use crate::Nature;

#[derive(Clone, Copy)]
pub struct CropView {
    pub entity: Crop,
    pub growth: f32,
    pub position: [f32; 2],
}

pub struct FarmerView {
    pub entity: Farmer,
    pub position: [f32; 2],
}

pub struct CreatureView {
    pub _entity: Creature,
}

pub struct InvaserView {
    _threat: f32,
}

pub struct BarrierView {
    id: BarrierId,
    kind: Shared<BarrierKind>,
    position: [f32; 2],
}

pub struct ContainerView {
    pub id: ContainerId,
    pub position: Position,
    pub owner: Owner,
}

#[derive(Clone)]
pub enum Owner {
    Stack(Stack),
    Hands(Farmer),
}

#[derive(Clone)]
pub struct ItemView {
    pub item: ItemId,
    pub container: ContainerId,
    pub kind: Shared<ItemKind>,
    pub quantity: u8,
}

#[derive(Clone)]
pub struct FoodView {
    pub item: ItemId,
    pub owner: Owner,
    pub quantity: u8,
    pub max_quantity: u8,
    pub position: [f32; 2],
}

pub struct TetherView {
    pub id: TetherId,
    pub length: f32,
    pub position: [f32; 2],
}

impl Nature {
    pub fn perceive_universe(&mut self, event: Universe) {
        match event {
            Universe::ActivityChanged { .. } => {}
            Universe::TreeAppeared { .. } => {}
            Universe::TreeVanished(_) => {}
            Universe::FarmlandAppeared {
                farmland, holes, ..
            } => {
                let mut holes_map = vec![0; 128 * 128];
                let barriers_map = vec![0; 128 * 128];
                for y in 0..holes.len() {
                    for x in 0..holes[0].len() {
                        let index = [x, y].fit(128);
                        holes_map[index] = holes[y][x];
                    }
                }
                self.holes_map.insert(farmland.space, holes_map);
                self.barriers_map.insert(farmland.space, barriers_map);
            }
            Universe::FarmlandVanished(_) => {}
            Universe::FarmerAppeared {
                farmer, position, ..
            } => {
                self.farmers.insert(
                    farmer,
                    FarmerView {
                        entity: farmer,
                        position,
                    },
                );
                self.containers.insert(
                    farmer.hands,
                    ContainerView {
                        id: farmer.hands,
                        position,
                        owner: Owner::Hands(farmer),
                    },
                );
                self.tethers.insert(
                    farmer.tether,
                    TetherView {
                        id: farmer.tether,
                        length: 5.0,
                        position,
                    },
                );
            }
            Universe::FarmerVanished(farmer) => {
                self.farmers.remove(&farmer);
                self.tethers.remove(&farmer.tether);
            }
            Universe::StackAppeared { stack, position } => {
                self.containers.insert(
                    stack.container,
                    ContainerView {
                        id: stack.container,
                        position,
                        owner: Owner::Stack(stack),
                    },
                );
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
            Universe::EquipmentAppeared { position, entity } => {
                if let Purpose::Tethering { tether } = entity.purpose {
                    self.tethers.insert(
                        tether,
                        TetherView {
                            id: tether,
                            length: 5.0,
                            position,
                        },
                    );
                }
            }
            Universe::EquipmentVanished(equipment) => {
                if let Purpose::Tethering { tether } = equipment.purpose {
                    self.tethers.remove(&tether);
                }
            }
            Universe::CreatureAppeared {
                entity,
                health,
                weight,
                position,
                hunger,
                farmland,
                behaviour,
                age,
            } => {
                self.creatures.push(CreatureView { _entity: entity });
                self.creature_agents.push(CreatureAgent {
                    entity,
                    farmland,
                    hunger,
                    health,
                    thinking: Thinking::default(),
                    targeting: Targeting::default(),
                    position,
                    interaction: 2.0,
                    radius: 5.0,
                    thirst: 0.0,
                    weight,
                    age,
                    colonization_date: 0.0,
                    daytime: 0.0,
                    behaviour,
                    timestamps: HashMap::new(),
                    cooldowns: HashMap::new(),
                    disabling: Default::default(),
                    tether: None,
                });
            }
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

    pub fn perceive_timing(&mut self, event: Timing) {
        match event {
            Timing::TimeUpdated {
                colonization_date, ..
            } => {
                self.colonization_date = colonization_date;
                for creature in self.creature_agents.iter_mut() {
                    creature.colonization_date = colonization_date;
                }
            }
            Timing::CalendarUpdated {
                times_of_day, id, ..
            } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.farmland.calendar == id {
                        creature.daytime = times_of_day;
                    }
                }
            }
        }
    }

    pub fn perceive_physics(&mut self, event: Physics) {
        match event {
            Physics::BodyPositionChanged { id, position, .. } => {
                for farmer in self.farmers.values_mut() {
                    if farmer.entity.body == id {
                        farmer.position = position;
                        self.tethers
                            .entry(farmer.entity.tether)
                            .and_modify(|tether| tether.position = position);
                        for container in self.containers.values_mut() {
                            if container.id == farmer.entity.hands {
                                container.position = position;
                            }
                        }
                        return;
                    }
                }
                for agent in self.creature_agents.iter_mut() {
                    if agent.entity.body == id {
                        agent.position = position;
                        return;
                    }
                }
            }
            Physics::BarrierCreated {
                space,
                position,
                key,
                id,
                active,
                ..
            } => {
                let barriers_map = self.barriers_map.get_mut(&space).expect("barriers_map");
                let kind = self.known.barriers.get(key).expect("barrier kind");
                let tiles = kind.tiles(position);
                self.barriers.insert(id, BarrierView { id, position, kind });
                for tile in tiles {
                    let index = tile.fit(128);
                    barriers_map[index] = if active { 1 } else { 0 };
                }
            }
            Physics::BarrierChanged { id, space, active } => {
                let barriers_map = self.barriers_map.get_mut(&space).expect("barriers_map");
                let barrier = self.barriers.get(&id).expect("barrier");
                let tiles = barrier.kind.tiles(barrier.position);
                for tile in tiles {
                    let index = tile.fit(128);
                    barriers_map[index] = if active { 1 } else { 0 };
                }
            }
            Physics::BarrierDestroyed {
                id,
                position,
                space,
                ..
            } => {
                let barriers_map = self.barriers_map.get_mut(&space).expect("barriers_map");
                let barrier = self.barriers.get(&id).expect("barrier");
                let tiles = barrier.kind.tiles(barrier.position);
                for tile in tiles {
                    let index = tile.fit(128);
                    barriers_map[index] = 0;
                }
                self.barriers.remove(&id);
            }
            Physics::SpaceUpdated { id, holes } => {
                let holes_map = self.holes_map.get_mut(&id).expect("holes_map");
                for y in 0..holes.len() {
                    for x in 0..holes.len() {
                        let index = [x, y].fit(128);
                        holes_map[index] = holes[y][x];
                    }
                }
            }
        }
    }

    pub fn perceive_raising(&mut self, event: Raising) {
        match event {
            Raising::AnimalChanged {
                id,
                hunger,
                thirst,
                age,
                weight,
            } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.hunger = hunger;
                        creature.thirst = thirst;
                        creature.age = age;
                        creature.weight = weight;
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
            Raising::BehaviourChanged { id, behaviour } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.behaviour = behaviour;
                        creature
                            .timestamps
                            .insert(behaviour, self.colonization_date);
                        break;
                    }
                }
            }
            Raising::BehaviourTriggered {
                id,
                behaviour,
                trigger,
            } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.behaviour = behaviour;
                        creature
                            .timestamps
                            .insert(behaviour, self.colonization_date);
                        creature.timestamps.insert(trigger, self.colonization_date);
                        break;
                    }
                }
            }
            Raising::AnimalTied { id, tether } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.tether = Some(tether);
                        break;
                    }
                }
            }
            Raising::AnimalUntied { id, .. } => {
                for creature in self.creature_agents.iter_mut() {
                    if creature.entity.animal == id {
                        creature.tether = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn perceive_inventory(&mut self, event: Inventory) {
        match event {
            Inventory::ContainerCreated { .. } => {}
            Inventory::ContainerDestroyed { id } => {
                self.items.remove(&id);
            }
            Inventory::ItemsAdded { items } => {
                for item in items {
                    let container = self.items.entry(item.container).or_insert(HashMap::new());
                    let kind = self.known.items.get(item.key).unwrap();
                    container.insert(
                        item.id,
                        ItemView {
                            item: item.id,
                            container: item.container,
                            kind,
                            quantity: item.quantity,
                        },
                    );
                }
            }
            Inventory::ItemRemoved { item, container } => {
                self.items.entry(container).and_modify(|items| {
                    items.remove(&item);
                });
            }
            Inventory::ItemQuantityChanged {
                id,
                container,
                quantity,
            } => {
                self.items.entry(container).and_modify(|items| {
                    items.entry(id).and_modify(|item| item.quantity = quantity);
                });
            }
        }
    }

    pub fn perceive(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::UniverseStream(events) => {
                    for event in events {
                        self.perceive_universe(event);
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
                        self.perceive_inventory(event);
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
                Event::TimingStream(events) => {
                    for event in events {
                        self.perceive_timing(event);
                    }
                }
            }
        }
    }
}
