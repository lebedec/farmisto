use crate::{Operation, Persist, Shared, Storage};
use log::{error, info};
use std::collections::HashMap;

pub struct Known<K> {
    knowledge: HashMap<usize, Shared<K>>,
    last_timestamp: i64,
}

impl<K> Default for Known<K> {
    fn default() -> Self {
        Self {
            knowledge: Default::default(),
            last_timestamp: -1,
        }
    }
}

impl<K, J> Known<K>
    where
        K: Persist<Id=J>,
        J: Into<usize>,
{
    #[inline]
    pub fn get(&self, key: J) -> Option<Shared<K>> {
        self.knowledge.get(&key.into()).cloned()
    }

    #[inline]
    pub fn get_unchecked(&self, key: usize) -> Shared<K> {
        self.knowledge.get(&key).unwrap().clone()
    }

    pub fn handle(&mut self, id: usize, operation: Operation, storage: Storage) {
        let connection = storage.connection();
        let table = std::any::type_name::<K>().split("::").last().unwrap();
        let mut statement = connection
            .prepare(&format!("select * from {} where id = ?", table))
            .unwrap();
        let mut rows = statement.query([self.last_timestamp]).unwrap();
        let row = rows.next().unwrap().unwrap();
        match operation {
            Operation::Insert | Operation::Update => match K::parse(row) {
                Ok(kind) => match self.knowledge.get_mut(&id) {
                    Some(reference) => { *reference.borrow_mut() = kind; }
                    None => { self.knowledge.insert(id, Shared::new(kind)); }
                }
                Err(error) => {
                    error!("Unable to parse {} row id={}, {}", table, id, error);
                }
            }
            Operation::Delete => { self.knowledge.remove(&id); }
        }
    }


    pub fn load(&mut self, storage: &Storage) {
        let connection = storage.connection();
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
