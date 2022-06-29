use crate::api::{Action, Event};
use crate::physics::PhysicsDomain;
use crate::planting::PlantingDomain;
pub use domains::*;
pub use model::*;
use rusqlite::Connection;

pub mod api;
mod domains;
mod model;
pub mod persistence;

pub struct Game {
    universe: Universe,
    physics: PhysicsDomain,
    planting: PlantingDomain,
}

impl Game {
    pub fn new() -> Self {
        Self {
            universe: Universe::default(),
            physics: PhysicsDomain::default(),
            planting: PlantingDomain::default(),
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

    pub fn look_around(&self) -> Vec<Event> {
        let mut _events = vec![];
        _events
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let _events = vec![];
        let connection = Connection::open("./assets/database.sqlite").unwrap();

        self.universe.load(&connection);

        self.physics.load(&connection);
        self.physics.update(time);

        self.planting.load(&connection);
        self.planting.update(time);

        _events
    }
}
