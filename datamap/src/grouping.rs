use crate::{Known, Persist, Shared, Storage};
use log::{error, info, warn};
use rusqlite::types::FromSql;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Grouping<T, G>
where
    T: Persist,
{
    groups: HashMap<G, Vec<T>>,
    last_timestamp: i64,
    last_id: usize,
    mapping: HashMap<T::Id, G>,
}

impl<T, G> Default for Grouping<T, G>
where
    T: Persist,
{
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
            last_timestamp: -1,
            last_id: 0,
            mapping: HashMap::new(),
        }
    }
}

impl<T, K, G, I, J> Grouping<T, G>
where
    T: Persist<Kind = Shared<K>, Id = I>,
    K: Persist<Id = J>,
    G: Clone + Hash + Eq + FromSql,
    I: Into<usize> + Hash + Eq + From<usize>,
    J: Into<usize>,
{
    #[inline]
    pub fn last_id(&self) -> usize {
        self.last_id
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
    pub fn get(&self, id: impl Into<I>) -> Option<&T> {
        let id: I = id.into();
        match self.mapping.get(&id) {
            None => None,
            Some(group) => {
                let id: usize = id.into();
                match self.groups.get(&group) {
                    Some(group) => group.iter().find(|item| item.entry_id() == id),
                    None => None,
                }
            }
        }
    }

    #[inline]
    pub fn get_mut(&mut self, id: impl Into<I>) -> Option<&mut T> {
        let id: I = id.into();
        match self.mapping.get(&id) {
            None => None,
            Some(group) => {
                let id: usize = id.into();
                match self.groups.get_mut(&group) {
                    Some(group) => group.iter_mut().find(|item| item.entry_id() == id),
                    None => None,
                }
            }
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

    pub fn load(&mut self, storage: &Storage, knowledge: &Known<K>) {
        let connection = storage.connection();
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
                            self.mapping.insert(id.into(), group_key);
                        }
                        Some(index) => {
                            group[index] = item;
                            updates += 1;
                            self.mapping.insert(id.into(), group_key);
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
        if inserts + updates + deletes > 0 {
            info!(
                "Load {}: {} inserted, {} updated, {} deleted, last timestamp is {}",
                table, inserts, updates, deletes, self.last_timestamp
            );
        }
    }

    pub fn dump() {}
}
