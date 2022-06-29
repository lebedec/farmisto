use crate::persistence::{Persist, Shared};
use log::{error, info};
use rusqlite::Connection;
use std::collections::HashMap;

pub struct Knowledge<K> {
    knowledge: HashMap<usize, Shared<K>>,
    last_timestamp: i64,
}

impl<K> Default for Knowledge<K> {
    fn default() -> Self {
        Self {
            knowledge: Default::default(),
            last_timestamp: -1,
        }
    }
}

impl<K: Persist> Knowledge<K> {
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
