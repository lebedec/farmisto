use crate::persistence::{Knowledge, Persist, Shared};
use log::{error, info, warn};
use rusqlite::Connection;

pub struct Collection<T> {
    items: Vec<T>,
    last_timestamp: i64,
    last_id: usize,
}

impl<T> Default for Collection<T> {
    fn default() -> Self {
        Self {
            items: vec![],
            last_timestamp: -1,
            last_id: 0,
        }
    }
}

impl<T, K> Collection<T>
where
    T: Persist<Kind = Shared<K>>,
    K: Persist,
{
    #[inline]
    pub fn next_id(&mut self) -> usize {
        self.last_id += 1;
        self.last_id
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

    pub fn load(&mut self, connection: &Connection, knowledge: &Knowledge<K>) {
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
            let kind = knowledge.get_unchecked(kind);
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
