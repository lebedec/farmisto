use std::collections::{HashMap, HashSet};

use datamap::{Storage};
pub use domains::*;

use crate::api::{Action, Event};
use crate::model::{Farmer, FarmerId, FarmerKey, FarmerKind, Farmland, FarmlandId, FarmlandKey, FarmlandKind, Tree, TreeId, TreeKey, TreeKind};
use crate::physics::{Physics, PhysicsDomain};
use crate::planting::PlantingDomain;

pub mod api;
mod domains;
pub mod math;
pub mod model;
mod data;
pub mod collections;

pub struct Game {
    pub universe: Universe,
    physics: PhysicsDomain,
    planting: PlantingDomain,
    storage: Storage,
}

impl Game {
    pub fn new(storage: Storage) -> Self {
        Self {
            universe: Universe::default(),
            physics: PhysicsDomain::default(),
            planting: PlantingDomain::default(),
            storage,
        }
    }

    pub fn perform_action(
        &mut self,
        player: &String,
        _action_id: usize,
        action: Action,
    ) -> Vec<Event> {
        let mut _events = vec![];
        match action {
            Action::DoSomething => {}
            Action::DoAnything { .. } => {}
            Action::MoveFarmer { destination } => {
                match self
                    .universe
                    .farmers
                    .iter()
                    .find(|farmer| &farmer.player == player)
                {
                    Some(farmer) => {
                        self.physics.move_body2(farmer.id.into(), destination);
                    }
                    None => {
                        // error framer not found, action_id error
                    }
                }
            }
        }
        _events
    }

    /// # Safety
    ///
    /// Relational database as source of data guarantees
    /// that domain objects exists while exist game model.
    /// So, we can unwrap references without check.
    pub fn look_around(&self, snapshot: UniverseSnapshot) -> Vec<Event> {
        let mut stream = vec![];

        for farmland in self.universe.farmlands.iter() {
            if snapshot.whole || snapshot.farmlands.contains(&farmland.id) {
                stream.push(Event::FarmlandAppeared {
                    id: farmland.id,
                    kind: farmland.kind.id,
                })
            }
        }
        let events = snapshot
            .farmlands_to_delete
            .into_iter()
            .map(Event::FarmlandVanished);
        stream.extend(events);

        for tree in self.universe.trees.iter() {
            if snapshot.whole || snapshot.trees.contains(&tree.id) {
                let barrier = self.physics.get_barrier(tree.id.into()).unwrap();
                let plant_kind = self.planting.known_plants.get(&tree.kind.plant).unwrap();
                stream.push(Event::BarrierHintAppeared {
                    id: barrier.id.0,
                    kind: barrier.kind.id.0,
                    position: barrier.position,
                    bounds: barrier.kind.bounds,
                });
                stream.push(Event::TreeAppeared {
                    id: tree.id,
                    kind: tree.kind.id,
                    position: barrier.position,
                    growth: plant_kind.growth,
                })
            }
        }
        let events = snapshot
            .trees_to_delete
            .into_iter()
            .map(Event::TreeVanished);
        stream.extend(events);

        for farmer in self.universe.farmers.iter() {
            if snapshot.whole || snapshot.farmers.contains(&farmer.id) {
                let body = self.physics.get_body(farmer.id.into()).unwrap();
                stream.push(Event::FarmerAppeared {
                    id: farmer.id,
                    kind: farmer.kind.id,
                    player: farmer.player.clone(),
                    position: body.position,
                })
            }
        }
        let events = snapshot
            .farmers_to_delete
            .into_iter()
            .map(Event::FarmerVanished);
        stream.extend(events);

        stream
    }

    pub fn load_game_full(&mut self) {
        self.load_game_knowledge();
        self.load_game_state();
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let mut events = vec![];
        for event in self.physics.update(time) {
            match event {
                Physics::BodyPositionChanged { id, position, .. } => {
                    if let Some(farmer) = self
                        .universe
                        .farmers
                        .iter()
                        .find(|farmer| id == farmer.id.into())
                    {
                        events.push(Event::FarmerMoved {
                            id: farmer.id,
                            position,
                        })
                    }
                }
            }
        }
        self.planting.update(time);
        events
    }
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub trees: HashMap<TreeKey, collections::Shared<TreeKind>>,
    pub farmlands: HashMap<FarmlandKey, collections::Shared<FarmlandKind>>,
    pub farmers: HashMap<FarmerKey, collections::Shared<FarmerKind>>,
}

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known: KnowledgeBase,
    pub farmlands: Vec<Farmland>,
    pub trees: Vec<Tree>,
    pub farmers: Vec<Farmer>,
}

#[derive(Default)]
pub struct UniverseSnapshot {
    pub whole: bool,
    pub farmlands: HashSet<FarmlandId>,
    pub farmlands_to_delete: HashSet<FarmlandId>,
    pub trees: HashSet<TreeId>,
    pub trees_to_delete: HashSet<TreeId>,
    pub farmers: HashSet<FarmerId>,
    pub farmers_to_delete: HashSet<FarmerId>,
}

impl UniverseSnapshot {
    pub fn whole() -> Self {
        let mut snapshot = UniverseSnapshot::default();
        snapshot.whole = true;
        snapshot
    }
}
