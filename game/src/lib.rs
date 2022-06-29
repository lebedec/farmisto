use crate::api::{Action, Event};
use crate::model::Universe;
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
    pub fn look_around(&self) -> Vec<Event> {
        let mut events = vec![];
        for tree in self.universe.trees.iter() {
            let barrier = self.physics.barriers.get(tree.id).unwrap();
            let plant_kind = self.planting.known_plants.get(tree.kind.plant).unwrap();
            events.push(Event::TreeAppeared {
                id: tree.id,
                kind: tree.kind.id,
                position: barrier.position,
                growth: plant_kind.growth,
            })
        }
        events
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let mut events = vec![];

        self.physics.load(&self.storage);
        self.planting.load(&self.storage);
        let changes = self.universe.load(&self.storage);
        events.extend(changes);

        self.physics.update(time);
        self.planting.update(time);

        events
    }
}
