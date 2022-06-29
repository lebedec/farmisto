use crate::persistence::{Known, Persist, Shared, Storage};
use log::{error, info, warn};

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

impl<T, K, J> Collection<T>
where
    T: Persist<Kind = Shared<K>>,
    K: Persist<Id = J>,
    J: Into<usize>,
{
    #[inline]
    pub fn last_id(&self) -> usize {
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

    pub fn load(&mut self, storage: &Storage, knowledge: &Known<K>) -> Changeset {
        let connection = storage.connection();
        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let mut statement = connection
            .prepare(&format!("select * from {} where timestamp > ?", table))
            .unwrap();
        let mut rows = statement.query([self.last_timestamp]).unwrap();

        let mut changeset = Changeset::new();
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
                        changeset.delete(id);
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
                            changeset.insert(id);
                        }
                        Some(index) => {
                            group[index] = item;
                            changeset.update(id);
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
        if changeset.changes() > 0 {
            info!(
                "Load {}: {} inserted, {} updated, {} deleted, last timestamp is {}",
                table,
                changeset.inserts.len(),
                changeset.updates.len(),
                changeset.deletes.len(),
                self.last_timestamp
            );
        }
        changeset
    }

    pub fn dump() {}
}

pub struct Changeset {
    pub inserts: Vec<usize>,
    pub updates: Vec<usize>,
    pub deletes: Vec<usize>,
}

impl Changeset {
    pub fn new() -> Self {
        Self {
            inserts: vec![],
            updates: vec![],
            deletes: vec![],
        }
    }

    #[inline]
    pub fn changes(&self) -> usize {
        self.inserts.len() + self.updates.len() + self.deletes.len()
    }

    #[inline]
    pub fn insert(&mut self, id: usize) {
        self.inserts.push(id);
    }

    #[inline]
    pub fn update(&mut self, id: usize) {
        self.updates.push(id);
    }

    #[inline]
    pub fn delete(&mut self, id: usize) {
        self.deletes.push(id);
    }
}
