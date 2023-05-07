use game::api::Event;
use game::math::VectorMath;
use game::model::{Creature, Crop, Universe};
use game::physics::Physics;

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
    _entity: Creature,
}

#[derive(Default, Clone, Copy)]
pub struct TileView {
    pub has_hole: bool,
    pub has_barrier: bool,
}

pub struct InvaserView {
    _threat: f32,
}

impl Nature {
    pub fn perceive(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::UniverseStream(events) => {
                    for event in events {
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
                            Universe::StackAppeared { .. } => {}
                            Universe::StackVanished(_) => {}
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
                            Universe::ItemsAppeared { .. } => {}
                            Universe::CreatureAppeared {
                                entity,
                                position,
                                hunger,
                                space,
                                ..
                            } => {
                                self.creatures.push(CreatureView { _entity: entity });
                                self.creature_agents.push(CreatureAgent {
                                    creature: entity,
                                    space,
                                    hunger,
                                    thinking: Thinking::default(),
                                    position,
                                    radius: 5,
                                })
                            }
                            Universe::CreatureEats { .. } => {}
                            Universe::CreatureVanished(_) => {}
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
                }
                Event::PhysicsStream(events) => {
                    for event in events {
                        match event {
                            Physics::BodyPositionChanged { id, position, .. } => {
                                for agent in self.creature_agents.iter_mut() {
                                    if agent.creature.body == id {
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
                            Physics::BarrierChanged { .. } => {}
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
                }
                Event::BuildingStream(_) => {}
                Event::InventoryStream(_) => {}
                Event::PlantingStream(_) => {}
                Event::RaisingStream(_) => {}
                Event::AssemblingStream(_) => {}
                Event::WorkingStream(_) => {}
                Event::LandscapingStream(_) => {}
                Event::TimingStream(_) => {}
            }
        }
    }
}
