use crate::physics::{BarrierId, BarrierKey, BodyId, BodyKey, SpaceId, SpaceKey};
use crate::planting::{LandId, LandKey, PlantId, PlantKey};
use datamap::{Collection, Id, Known, Persisted, Shared, Storage};
use log::info;
use std::collections::HashSet;

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known_trees: Known<TreeKind>,
    pub known_farmlands: Known<FarmlandKind>,
    pub known_farmers: Known<FarmerKind>,
    pub farmlands: Collection<Farmland>,
    pub trees: Collection<Tree>,
    pub farmers: Collection<Farmer>,
}

#[derive(Default)]
pub struct UniverseSnapshot {
    pub whole: bool,
    pub farmlands: HashSet<FarmlandId>,
    pub farmlands_to_delete: HashSet<FarmlandId>,
    pub trees: HashSet<TreeId>,
    pub trees_to_delete: HashSet<TreeId>,
    pub farmers: HashSet<FarmerId>,
    pub farmers_to_delete: HashSet<FarmerId>,
}

impl UniverseSnapshot {
    pub fn whole() -> Self {
        let mut snapshot = UniverseSnapshot::default();
        snapshot.whole = true;
        snapshot
    }
}

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmerKey(usize);

#[derive(Persisted)]
pub struct FarmerKind {
    pub id: FarmerKey,
    pub name: String,
    pub body: BodyKey,
}

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct FarmerId(usize);

impl From<FarmerId> for BodyId {
    fn from(id: FarmerId) -> Self {
        id.0.into()
    }
}

#[derive(Persisted)]
pub struct Farmer {
    pub id: FarmerId,
    pub kind: Shared<FarmerKind>,
    pub player: String,
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
        self.known_farmers.load(storage);

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

        let changeset = self.farmers.load(storage, &self.known_farmers);
        // todo: automate changeset conversion to universe snapshot
        for id in changeset.inserts {
            snapshot.farmers.insert(id.into());
        }
        for id in changeset.updates {
            snapshot.farmers.insert(id.into());
        }
        for id in changeset.deletes {
            snapshot.farmers_to_delete.insert(id.into());
        }

        let next_id = *[
            self.id,
            self.farmlands.last_id(),
            self.trees.last_id(),
            self.farmers.last_id(),
        ]
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
