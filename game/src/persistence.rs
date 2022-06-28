pub use game_derive::{group, Domain, Persisted};
use log::{error, info, warn};
use rusqlite::{Connection, Row, Statement};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::hash_map::{Values, ValuesMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

pub trait Persist: Sized {
    fn columns() -> Vec<String>;
    fn bind(&self, statement: &mut Statement) -> rusqlite::Result<()>;
    fn parse(row: &Row) -> Result<Self, rusqlite::Error>;
}

pub struct Grouping<G, T, K> {
    knowledge: HashMap<usize, Rc<K>>,
    groups: HashMap<G, Vec<T>>,
    fallback: Vec<T>,
}

impl<G: Debug + Hash + Eq, T, K> Grouping<G, T, K> {
    pub fn new() -> Self {
        Self {
            knowledge: HashMap::new(),
            groups: HashMap::new(),
            fallback: vec![],
        }
    }

    pub fn load() {
        todo!()
    }

    #[inline]
    pub fn iter(&self, group: G) -> &Vec<T> {
        match self.groups.get(&group) {
            Some(values) => values,
            None => {
                warn!(
                    "Attempt to get the {} values of a non-existent group '{:?}'",
                    std::any::type_name::<T>(),
                    group
                );
                &self.fallback
            }
        }
    }

    pub fn iter_mut(&mut self, group: G) -> &mut Vec<T> {
        match self.groups.get_mut(&group) {
            Some(values) => values,
            None => {
                warn!(
                    "Attempt to get the {} values of a non-existent group '{:?}'",
                    std::any::type_name::<T>(),
                    group
                );
                &mut self.fallback
            }
        }
    }
}

pub struct MutableGrouping<G, T> {
    last_timestamp: i64,
    last_group_id: usize,
    last_id: usize,
    groups: HashMap<usize, (G, HashMap<usize, T>)>,
    groups2: Vec<(G, Vec<T>)>,
}

impl<G, T: Persist> MutableGrouping<G, T> {
    pub fn new() -> Self {
        Self {
            last_timestamp: -1,
            last_group_id: 0,
            last_id: 0,
            groups: Default::default(),
            groups2: vec![],
        }
    }

    #[inline]
    pub fn next_id(&mut self) -> usize {
        self.last_id += 1;
        self.last_id
    }

    #[inline]
    pub fn iter(&self) -> Values<usize, (G, HashMap<usize, T>)> {
        self.groups.values()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ValuesMut<usize, (G, HashMap<usize, T>)> {
        self.groups.values_mut()
    }

    pub fn iter_groups(&mut self) -> &mut Vec<(G, Vec<T>)> {
        &mut self.groups2
    }

    #[inline]
    pub fn get(&self, group: usize, id: usize) -> Option<&T> {
        self.groups.get(&group).unwrap().1.get(&id)
    }

    #[inline]
    pub fn get_mut(&mut self, group: usize, id: usize) -> Option<&mut T> {
        self.groups.get_mut(&group).unwrap().1.get_mut(&id)
    }
}

#[derive(Debug)]
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

    pub fn load<A, R, E>(&mut self, connection: &Connection, insert: A, remove: R) -> Vec<E>
    where
        A: Fn(&T) -> E,
        R: Fn(usize) -> E,
    {
        let mut events = vec![];
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
                events.push(remove(id));
                self.items.remove(&id);
                deletes += 1;
            } else {
                match T::parse(row) {
                    Ok(item) => {
                        events.push(insert(&item));
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
        events
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
    pub fn get(&self, id: usize) -> Option<&T> {
        self.items.get(&id)
    }

    #[inline]
    pub fn get_unchecked(&self, id: usize) -> &T {
        self.items.get(&id).unwrap()
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
