use crate::api::{Action, Event};
use crate::physics::{Physics, PhysicsDomain};
use crate::shapes::ShapesDomain;
pub use domains::*;
pub use model::*;
use rusqlite::Connection;

pub mod api;
mod domains;
mod model;
pub mod persistence;

pub struct Game {
    shapes: ShapesDomain,
    physics: PhysicsDomain,
    time: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            shapes: ShapesDomain::new(),
            physics: PhysicsDomain::new(),
            time: 0.0,
        }
    }

    pub fn perform_action(&mut self, action_id: usize, action: Action) -> Vec<Event> {
        let mut events = vec![];
        match action {
            Action::DoSomething => {
                let my_events = self.shapes.create_triangle();
                events.extend(my_events.into_iter().map(|e| Event::ShapesEvents(e)));
            }
            Action::DoAnything { .. } => {}
        }
        events
    }

    pub fn look_around(&self) -> Vec<Event> {
        let mut events = vec![];
        let shapes = self.shapes.look_around();
        events.extend(shapes.into_iter().map(|e| Event::ShapesEvents(e)));
        events
    }

    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let mut events = vec![];
        let connection = Connection::open("./assets/database.sqlite").unwrap();
        let state_events = self.shapes.load_state(&connection);
        events.extend(state_events.into_iter().map(|e| Event::ShapesEvents(e)));
        self.time += time;
        if self.time > 5.0 {
            // events.push(Event::TrianglesSumChanged { sum: self.time });
            self.shapes.update(self.time);
            self.time = 0.0;
        }

        let physics_events = self.physics.update(time);
        events.extend(physics_events.into_iter().map(|e| Event::PhysicsEvents(e)));
        // for event in physics_events {
        //     match event {
        //         Physics::BodyPositionChanged { id, position } => {
        //             if origin == "entity" {
        //                 events.push(Event::EntityPositionChanged {
        //                     id,
        //                     position
        //                 })
        //             }
        //             if origin == "wall" {
        //                 events.push(Event::WallPositionChanged {
        //                     id,
        //                     position
        //                 })
        //             }
        //         }
        //     }
        // }

        events
    }
}
