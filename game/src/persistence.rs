use log::{error, info};
use rusqlite::{Connection, Row, Statement};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::hash_map::{Values, ValuesMut};
use std::collections::HashMap;

pub use game_derive::{Domain, Persisted};

pub trait Persist: Sized {
    fn columns() -> Vec<String>;
    fn bind(&self, statement: &mut Statement) -> rusqlite::Result<()>;
    fn parse(row: &Row) -> Result<Self, rusqlite::Error>;
}

pub struct Mutable<T> {
    last_entry: i64,
    last_id: usize,
    items: HashMap<usize, T>,
}

impl<T: Persist> Mutable<T> {
    pub fn new() -> Self {
        Self {
            last_entry: -1,
            last_id: 0,
            items: HashMap::new(),
        }
    }

    #[inline]
    pub fn next_id(&mut self) -> usize {
        self.last_id += 1;
        self.last_id
    }

    #[inline]
    pub fn insert(&mut self, id: usize, value: T) {
        self.items.insert(id, value);
    }

    #[inline]
    pub fn remove(&mut self, id: usize) {
        self.items.remove(&id);
    }

    #[inline]
    pub fn iter(&self) -> Values<usize, T> {
        self.items.values()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ValuesMut<usize, T> {
        self.items.values_mut()
    }

    #[inline]
    pub fn get(&self, id: usize) -> Option<&T> {
        self.items.get(&id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    pub fn load(&mut self, connection: &Connection) {
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
            if id > self.last_id {
                self.last_id = id;
            }
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

    pub fn dump(&mut self, connection: &Connection) {
        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let mut columns = vec!["deleted".to_string()];
        columns.extend(T::columns());
        let values = vec!["?"; columns.len()].join(",");
        let columns = columns.join(",");
        let mut statement = connection
            .prepare(&format!(
                "insert into {} ({}) values ({})",
                table, columns, values
            ))
            .unwrap();
        for item in self.items.values() {
            statement.raw_bind_parameter(1, false).unwrap();
            item.bind(&mut statement).unwrap();
            statement.raw_execute().unwrap();
        }
    }

    pub fn clean(&mut self, _connection: &Connection) {}
}

pub struct Readonly<T> {
    last_entry: i64,
    items: HashMap<usize, T>,
}

impl<T: Persist> Readonly<T> {
    pub fn new() -> Self {
        Self {
            last_entry: -1,
            items: HashMap::new(),
        }
    }

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

pub fn parse_json_value<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

pub fn to_json_value<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap()
}
