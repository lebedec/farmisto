use crate::persistence::{Collection, Id, Known, Persisted, Shared, Storage};
use crate::physics::{BarrierId, BarrierKey, SpaceId, SpaceKey};
use crate::planting::{LandId, LandKey, PlantId, PlantKey};
use log::info;
use std::collections::hash_set::IntoIter;
use std::collections::HashSet;

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known_trees: Known<TreeKind>,
    pub known_farmlands: Known<FarmlandKind>,
    pub farmlands: Collection<Farmland>,
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
    pub farmlands: HashSet<FarmlandId>,
    pub farmlands_to_delete: HashSet<FarmlandId>,
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

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmlandKey(usize);

#[derive(Persisted)]
pub struct FarmlandKind {
    pub id: FarmlandKey,
    pub name: String,
    pub space: SpaceKey,
    pub land: LandKey,
}

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmlandId(usize);

impl From<FarmlandId> for SpaceId {
    fn from(id: FarmlandId) -> Self {
        id.0.into()
    }
}

impl From<FarmlandId> for LandId {
    fn from(id: FarmlandId) -> Self {
        id.0.into()
    }
}

#[derive(Persisted)]
pub struct Farmland {
    pub id: FarmlandId,
    pub kind: Shared<FarmlandKind>,
}

impl Universe {
    pub fn load(&mut self, storage: &Storage) -> UniverseSnapshot {
        let mut snapshot = UniverseSnapshot::default();

        self.known_farmlands.load(storage);
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
        let changeset = self.farmlands.load(storage, &self.known_farmlands);
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

        let next_id = *[self.id, self.farmlands.last_id(), self.trees.last_id()]
            .iter()
            .max()
            .unwrap();
        if next_id != self.id {
            info!("Advance id value from {} to {}", self.id, next_id);
            self.id = next_id;
        }

        snapshot
    }
}
