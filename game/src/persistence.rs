pub use game_derive::{Domain, Persisted};
use log::{error, info, warn};
use rusqlite::types::FromSql;
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
    type Kind;

    fn entry_id(&self) -> usize {
        unimplemented!()
    }

    fn columns() -> Vec<String>;

    fn bind(&self, statement: &mut Statement) -> rusqlite::Result<()>;

    #[allow(unused_variables)]
    fn parse(row: &Row) -> Result<Self, rusqlite::Error> {
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn parse_known(row: &Row, kind: Self::Kind) -> Result<Self, rusqlite::Error> {
        unimplemented!()
    }

    fn group() -> String {
        unimplemented!()
    }
}

pub struct Grouping<G, T, K> {
    knowledge: Knowledge<K>,
    groups: HashMap<G, Vec<T>>,
    last_timestamp: i64,
    last_id: usize,
}

impl<G, T, K> Grouping<G, T, K>
where
    G: Clone + Debug + Hash + Eq + FromSql,
    T: Persist<Kind = Rc<K>> + Debug,
    K: Persist + Debug,
{
    pub fn new() -> Self {
        Self {
            knowledge: Knowledge::new(),
            groups: HashMap::new(),
            last_timestamp: -1,
            last_id: 0,
        }
    }

    #[inline]
    pub fn next_id(&mut self) -> usize {
        self.last_id += 1;
        self.last_id
    }

    #[inline]
    pub fn get_kind(&self, id: usize) -> Option<Rc<K>> {
        self.knowledge.get(id)
    }

    #[inline]
    pub fn insert(&mut self, group: G, item: T) {
        self.groups.get_mut(&group).unwrap().push(item);
    }

    #[inline]
    pub fn iter(&self, group: G) -> Option<&Vec<T>> {
        self.groups.get(&group)
    }

    #[inline]
    pub fn iter_mut(&mut self, group: G) -> Option<&mut Vec<T>> {
        self.groups.get_mut(&group)
    }

    pub fn load(&mut self, connection: &Connection) {
        self.knowledge.load(connection);

        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let group_field = T::group();
        let mut statement = connection
            .prepare(&format!("select * from {} where timestamp > ?", table))
            .unwrap();
        let mut rows = statement.query([self.last_timestamp]).unwrap();
        let mut inserts = 0;
        let mut updates = 0;
        let mut deletes = 0;
        while let Some(row) = rows.next().unwrap() {
            let id: usize = row.get("id").unwrap();
            if id > self.last_id {
                self.last_id = id;
            }
            let timestamp: i64 = row.get("timestamp").unwrap();
            let deleted: bool = row.get("deleted").unwrap();
            let kind: usize = row.get("kind").unwrap();
            let kind = self.knowledge.get_unchecked(kind);
            let group_key: G = row.get(&group_field[..]).unwrap();
            if !self.groups.contains_key(&group_key) {
                self.groups.insert(group_key.clone(), vec![]);
            }
            let group = self.groups.get_mut(&group_key).unwrap();
            if deleted {
                match group.iter().position(|item| item.entry_id() == id) {
                    Some(index) => {
                        group.remove(index);
                        deletes += 1;
                    }
                    None => {
                        warn!(
                            "Unable to remove {} with id={}, not found in runtime",
                            table, id
                        );
                    }
                }
            } else {
                match T::parse_known(row, kind) {
                    Ok(item) => match group.iter().position(|item| item.entry_id() == id) {
                        None => {
                            println!("INSERT ITEM: {:?}", item);
                            group.push(item);
                            inserts += 1;
                        }
                        Some(index) => {
                            println!("UPDATE ITEM[{}]: {:?}", index, item);
                            group[index] = item;
                            updates += 1;
                        }
                    },
                    Err(error) => {
                        error!("Unable to parse {} row id={}, {}", table, id, error);
                        break;
                    }
                }
            }
            self.last_timestamp = timestamp;
        }
        if updates + deletes > 0 {
            info!(
                "Load {}: {} inserted, {} updated, {} deleted, last timestamp is {}",
                table, inserts, updates, deletes, self.last_timestamp
            );
        }
    }

    pub fn dump() {}
}

pub struct Knowledge<K> {
    knowledge: HashMap<usize, Rc<K>>,
    last_timestamp: i64,
}

impl<K: Persist + Debug> Knowledge<K> {
    pub fn new() -> Self {
        Self {
            knowledge: Default::default(),
            last_timestamp: -1,
        }
    }

    #[inline]
    pub fn get(&self, key: usize) -> Option<Rc<K>> {
        self.knowledge.get(&key).cloned()
    }

    #[inline]
    pub fn get_unchecked(&self, key: usize) -> Rc<K> {
        self.knowledge.get(&key).unwrap().clone()
    }

    pub fn load(&mut self, connection: &Connection) {
        let table = std::any::type_name::<K>().split("::").last().unwrap();
        let mut statement = connection
            .prepare(&format!("select * from {} where timestamp > ?", table))
            .unwrap();
        let mut rows = statement.query([self.last_timestamp]).unwrap();
        let mut updates = 0;
        let mut deletes = 0;
        while let Some(row) = rows.next().unwrap() {
            let id: usize = row.get("id").unwrap();
            let timestamp: i64 = row.get("timestamp").unwrap();
            let deleted: bool = row.get("deleted").unwrap();
            if deleted {
                self.knowledge.remove(&id);
                deletes += 1;
            } else {
                match K::parse(row) {
                    Ok(kind) => {
                        println!("KIND: {:?}", kind);
                        self.knowledge.insert(id, Rc::new(kind));

                        updates += 1;
                    }
                    Err(error) => {
                        error!("Unable to parse {} row id={}, {}", table, id, error);
                        break;
                    }
                }
            }
            self.last_timestamp = timestamp;
        }
        if updates + deletes > 0 {
            info!(
                "Load {}: {} updated, {} deleted, now {}, last timestamp is {}",
                table,
                updates,
                deletes,
                self.knowledge.len(),
                self.last_timestamp
            );
        }
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
