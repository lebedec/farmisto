pub use game_derive::{Domain, Id, Persisted};
use log::{error, info, warn};
use rusqlite::types::FromSql;
use rusqlite::{Connection, Row, Statement};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::Deref;
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

pub struct Grouping<T, G> {
    groups: HashMap<G, Vec<T>>,
    last_timestamp: i64,
    last_id: usize,
}

impl<T, K, G> Grouping<T, G>
where
    T: Persist<Kind = Shared<K>> + Debug,
    K: Persist + Debug,
    G: Clone + Debug + Hash + Eq + FromSql,
{
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            last_timestamp: -1,
            last_id: 0,
        }
    }

    #[inline]
    pub fn next_id<I: From<usize>>(&mut self) -> I {
        self.last_id += 1;
        self.last_id.into()
    }

    #[inline]
    pub fn insert(&mut self, group: G, item: T) {
        self.groups.get_mut(&group).unwrap().push(item);
    }

    #[inline]
    pub fn remove_at(&mut self, group: G, index: usize) -> T {
        self.groups.get_mut(&group).unwrap().remove(index)
    }

    #[inline]
    pub fn get<I: Into<usize> + Copy>(&mut self, group: G, id: I) -> Option<&T> {
        match self.groups.get(&group) {
            Some(group) => group.iter().find(|item| item.entry_id() == id.into()),
            None => None,
        }
    }

    #[inline]
    pub fn get_mut<I: Into<usize> + Copy>(&mut self, group: G, id: I) -> Option<&mut T> {
        match self.groups.get_mut(&group) {
            Some(group) => group.iter_mut().find(|item| item.entry_id() == id.into()),
            None => None,
        }
    }

    #[inline]
    pub fn iter(&self, group: G) -> Option<&Vec<T>> {
        self.groups.get(&group)
    }

    #[inline]
    pub fn iter_mut(&mut self, group: G) -> Option<&mut Vec<T>> {
        self.groups.get_mut(&group)
    }

    pub fn load(&mut self, connection: &Connection, knowledge: &Knowledge<K>) {
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
            let kind = knowledge.get_unchecked(kind);
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

pub struct Collection<T, K> {
    knowledge: Knowledge<K>,
    items: Vec<T>,
    last_timestamp: i64,
    last_id: usize,
}

impl<T, K> Collection<T, K>
where
    T: Persist<Kind = Shared<K>> + Debug,
    K: Persist + Debug,
{
    pub fn new() -> Self {
        Self {
            knowledge: Knowledge::new(),
            items: vec![],
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
    pub fn get_kind(&self, id: usize) -> Option<Shared<K>> {
        self.knowledge.get(id)
    }

    #[inline]
    pub fn insert(&mut self, item: T) {
        self.items.push(item);
    }

    #[inline]
    pub fn iter(&self) -> &Vec<T> {
        &self.items
    }

    #[inline]
    pub fn iter_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }

    pub fn load(&mut self, connection: &Connection) {
        self.knowledge.load(connection);

        let table = std::any::type_name::<T>().split("::").last().unwrap();
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
            let group = &mut self.items;
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
    knowledge: HashMap<usize, Shared<K>>,
    last_timestamp: i64,
}

pub struct Shared<T> {
    inner: Rc<RefCell<T>>,
}

impl<T: Debug> Debug for Shared<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    #[inline]
    pub fn borrow_mut(&mut self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.as_ptr() }
    }
}

impl<K: Persist + Debug> Knowledge<K> {
    pub fn new() -> Self {
        Self {
            knowledge: Default::default(),
            last_timestamp: -1,
        }
    }

    #[inline]
    pub fn get(&self, key: usize) -> Option<Shared<K>> {
        self.knowledge.get(&key).cloned()
    }

    #[inline]
    pub fn get_unchecked(&self, key: usize) -> Shared<K> {
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
                        match self.knowledge.get_mut(&id) {
                            Some(reference) => {
                                *reference.borrow_mut() = kind;
                            }
                            None => {
                                self.knowledge.insert(id, Shared::new(kind));
                            }
                        }
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

pub fn parse_json_value<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

pub fn to_json_value<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap()
}
