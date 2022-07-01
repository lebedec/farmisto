use crate::persistence::{Collection, Id, Known, Persisted, Shared, Storage};
use crate::physics::{BarrierId, BarrierKey};
use crate::planting::{PlantId, PlantKey};
use log::info;
use std::collections::hash_set::IntoIter;
use std::collections::HashSet;

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known_trees: Known<TreeKind>,
    pub trees: Collection<Tree>,
}

#[derive(Default)]
pub struct Set<T> {
    inner: HashSet<T>,
}

impl<T> Set<T> {
    pub fn new(inner: HashSet<T>) -> Self {
        Self { inner }
    }

    pub fn map(self) -> IntoIter<T> {
        self.inner.into_iter()
    }
}

#[derive(Default)]
pub struct UniverseSnapshot {
    pub whole: bool,
    pub trees: HashSet<TreeId>,
    pub trees_to_delete: HashSet<TreeId>,
}

impl UniverseSnapshot {
    pub fn whole() -> Self {
        let mut snapshot = UniverseSnapshot::default();
        snapshot.whole = true;
        snapshot
    }
}

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct TreeKey(usize);

#[derive(Persisted)]
pub struct TreeKind {
    pub id: TreeKey,
    pub name: String,
    pub barrier: BarrierKey,
    pub plant: PlantKey,
}

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct TreeId(usize);

impl From<TreeId> for BarrierId {
    fn from(id: TreeId) -> Self {
        id.0.into()
    }
}

impl From<TreeId> for PlantId {
    fn from(id: TreeId) -> Self {
        id.0.into()
    }
}

#[derive(Persisted)]
pub struct Tree {
    pub id: TreeId,
    pub kind: Shared<TreeKind>,
}

impl Universe {
    pub fn load(&mut self, storage: &Storage) -> UniverseSnapshot {
        let mut snapshot = UniverseSnapshot::default();

        self.known_trees.load(storage);
        let changeset = self.trees.load(storage, &self.known_trees);
        // todo: automate changeset conversion to universe snapshot
        for id in changeset.inserts {
            snapshot.trees.insert(id.into());
        }
        for id in changeset.updates {
            snapshot.trees.insert(id.into());
        }
        for id in changeset.deletes {
            snapshot.trees_to_delete.insert(id.into());
        }

        let next_id = *[self.id, self.trees.last_id()].iter().max().unwrap();
        if next_id != self.id {
            info!("Advance id value from {} to {}", self.id, next_id);
            self.id = next_id;
        }

        snapshot
    }
}
