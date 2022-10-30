use crate::api::{Action, Event};
use crate::model::{
    Farmer, FarmerId, FarmerKind, Farmland, FarmlandId, FarmlandKind, Tree, TreeId, TreeKind,
};
use crate::physics::{Physics, PhysicsDomain};
use crate::planting::PlantingDomain;
use datamap::{Collection, Known, Storage};
pub use domains::*;
use log::info;
use std::collections::HashSet;

pub mod api;
mod domains;
pub mod math;
pub mod model;
mod data;

pub struct Game {
    universe: Universe,
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
                let barrier = self.physics.barriers.get(tree.id).unwrap();
                let plant_kind = self.planting.known_plants.get(tree.kind.plant).unwrap();
                stream.push(Event::BarrierHintAppeared {
                    id: barrier.id.into(),
                    kind: barrier.kind.id.into(),
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
                let body = self.physics.bodies.get(farmer.id).unwrap();
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

    pub fn hot_reload(&mut self) -> Vec<Event> {
        let mut events = vec![];

        self.physics.load(&self.storage);
        self.planting.load(&self.storage);
        let changes = self.universe.load(&self.storage);
        events.extend(self.look_around(changes));

        events
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
    pub trees: Known<TreeKind>,
    pub farmlands: Known<FarmlandKind>,
    pub farmers: Known<FarmerKind>,
}

impl KnowledgeBase {
    pub fn load(&mut self, storage: &Storage) {
        self.trees.load(storage);
        self.farmlands.load(storage);
        self.farmers.load(storage);
    }
}

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known: KnowledgeBase,
    pub farmlands: Collection<Farmland>,
    pub trees: Collection<Tree>,
    pub farmers: Collection<Farmer>,
}

impl Universe {
    pub fn load(&mut self, storage: &Storage) -> UniverseSnapshot {
        let mut snapshot = UniverseSnapshot::default();

        self.known.load(storage);

        let changeset = self.trees.load(storage, &self.known.trees);
        // todo: automate changeset conversion to universe snapshot
        for id in changeset.inserts {
            snapshot.trees.insert(id.into());
        }
        for id in changeset.updates {
            snapshot.trees.insert(id.into());
        }
        for id in changeset.deletes {
            snapshot.trees_to_delete.insert(id.into());
        }

        let changeset = self.farmlands.load(storage, &self.known.farmlands);
        // todo: automate changeset conversion to universe snapshot
        for id in changeset.inserts {
            snapshot.farmlands.insert(id.into());
        }
        for id in changeset.updates {
            snapshot.farmlands.insert(id.into());
        }
        for id in changeset.deletes {
            snapshot.farmlands_to_delete.insert(id.into());
        }

        let changeset = self.farmers.load(storage, &self.known.farmers);
        // todo: automate changeset conversion to universe snapshot
        for id in changeset.inserts {
            snapshot.farmers.insert(id.into());
        }
        for id in changeset.updates {
            snapshot.farmers.insert(id.into());
        }
        for id in changeset.deletes {
            snapshot.farmers_to_delete.insert(id.into());
        }

        let next_id = *[
            self.id,
            self.farmlands.last_id(),
            self.trees.last_id(),
            self.farmers.last_id(),
        ]
        .iter()
        .max()
        .unwrap();
        if next_id != self.id {
            info!("Advance id value from {} to {}", self.id, next_id);
            self.id = next_id;
        }

        snapshot
    }
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
