use crate::persistence::{Knowledge, Persist, Shared};
use log::{error, info, warn};
use rusqlite::types::FromSql;
use rusqlite::Connection;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Grouping<T, G> {
    groups: HashMap<G, Vec<T>>,
    last_timestamp: i64,
    last_id: usize,
}

impl<T, G> Default for Grouping<T, G> {
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
            last_timestamp: -1,
            last_id: 0,
        }
    }
}

impl<T, K, G> Grouping<T, G>
where
    T: Persist<Kind = Shared<K>>,
    K: Persist,
    G: Clone + Hash + Eq + FromSql,
{
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
                            group.push(item);
                            inserts += 1;
                        }
                        Some(index) => {
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
