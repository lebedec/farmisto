use crate::shapes::Shapes;
pub use domains::*;
use rusqlite::Connection;

pub mod api;
mod domains;
pub mod persistence;

pub struct Game {
    shapes: Shapes,
}

impl Game {
    pub fn new() -> Self {
        Self {
            shapes: Shapes::new(),
        }
    }

    pub fn update(&mut self, time: f32) {
        let connection = Connection::open("./assets/database.sqlite").unwrap();
        self.shapes.load(&connection);
        self.shapes.update(time);
        self.shapes.dump(&connection);
    }
}
