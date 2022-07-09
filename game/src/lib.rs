use crate::api::{Action, Event};
use crate::model::{Universe, UniverseSnapshot};
use crate::persistence::Storage;
use crate::physics::PhysicsDomain;
use crate::planting::PlantingDomain;
pub use domains::*;

pub mod api;
mod domains;
pub mod model;
pub mod persistence;

pub struct Game {
    universe: Universe,
    physics: PhysicsDomain,
    planting: PlantingDomain,
    storage: Storage,
}

impl Game {
    pub fn new() -> Self {
        Self {
            universe: Universe::default(),
            physics: PhysicsDomain::default(),
            planting: PlantingDomain::default(),
            storage: Storage::open("./assets/database.sqlite").unwrap(),
        }
    }

    pub fn perform_action(&mut self, _action_id: usize, action: Action) -> Vec<Event> {
        let mut _events = vec![];
        match action {
            Action::DoSomething => {}
            Action::DoAnything { .. } => {}
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

        stream
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let mut events = vec![];

        self.physics.load(&self.storage);
        self.planting.load(&self.storage);
        let changes = self.universe.load(&self.storage);
        events.extend(self.look_around(changes));

        self.physics.update(time);
        self.planting.update(time);

        events
    }
}
