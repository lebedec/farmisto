use crate::shapes::{QuadKind, Triangle, TriangleKind};
pub use domains::*;
use log::{error, info};
use rusqlite::{Connection, Error, Row};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

mod domains;
pub mod knowledge;

struct Readonly<T> {
    last_entry: i64,
    items: HashMap<usize, T>,
}

impl<T> Readonly<T> {
    pub fn new() -> Self {
        Self {
            last_entry: -1,
            items: HashMap::new(),
        }
    }
}

trait Persist: Sized {
    fn parse(row: &Row) -> Result<Self, rusqlite::Error>;
}

impl Persist for TriangleKind {
    fn parse(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            name: row.get("name")?,
        })
    }
}

impl Persist for QuadKind {
    fn parse(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            name: row.get("name")?,
        })
    }
}

fn parse_json_value<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

impl Persist for Triangle {
    fn parse(row: &Row) -> Result<Self, Error> {
        Ok(Self {
            id: row.get("id")?,
            kind: row.get("kind")?,
            position: parse_json_value(row.get("position")?),
        })
    }
}

impl<T: Persist> Readonly<T> {
    #[inline]
    pub fn _get(&self, id: usize) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn update(&mut self, connection: &Connection) {
        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let mut statement = connection
            .prepare(&format!(
                "select * from {} where entry > ? order by entry",
                table
            ))
            .unwrap();
        let mut rows = statement.query([self.last_entry]).unwrap();
        let mut updates = 0;
        let mut deletes = 0;
        while let Some(row) = rows.next().unwrap() {
            let entry = row.get("entry").unwrap();
            let id = row.get("id").unwrap();
            let deleted: bool = row.get("deleted").unwrap();
            if deleted {
                self.items.remove(&id);
                deletes += 1;
            } else {
                match T::parse(row) {
                    Ok(item) => {
                        self.items.insert(id, item);
                        updates += 1;
                    }
                    Err(error) => {
                        error!("Unable to parse {} row entry={}, {}", table, entry, error);
                        break;
                    }
                }
            }
            self.last_entry = entry;
        }
        if updates + deletes > 0 {
            info!(
                "Synchronize {}: {} updated, {} deleted, now {}, last entry is {}",
                table,
                updates,
                deletes,
                self.items.len(),
                self.last_entry
            );
        }
    }
}

pub struct Game {
    triangle_kinds: Readonly<TriangleKind>,
    triangles: Readonly<Triangle>,
    quad_kinds: Readonly<QuadKind>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            triangle_kinds: Readonly::new(),
            triangles: Readonly::new(),
            quad_kinds: Readonly::new(),
        }
    }

    pub fn update(&mut self) {
        let connection = Connection::open("./assets/database.sqlite").unwrap();
        self.triangle_kinds.update(&connection);
        self.quad_kinds.update(&connection);
        self.triangles.update(&connection);
    }
}
