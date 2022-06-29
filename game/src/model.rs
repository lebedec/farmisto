use crate::persistence::{Collection, Id, Known, Persisted, Shared, Storage};
use crate::physics::{BarrierId, BarrierKey};
use crate::planting::{PlantId, PlantKey};
use crate::Event;
use log::info;

#[derive(Default)]
pub struct Universe {
    pub id: usize,
    pub known_trees: Known<TreeKind>,
    pub trees: Collection<Tree>,
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
    pub fn load(&mut self, storage: &Storage) -> Vec<Event> {
        let mut events = vec![];

        self.known_trees.load(storage);
        let changeset = self.trees.load(storage, &self.known_trees);
        // todo: reuse game look around
        for id in changeset.inserts {
            events.push(Event::TreeUpdated { id: id.into() })
        }
        for id in changeset.updates {
            events.push(Event::TreeUpdated { id: id.into() })
        }
        for id in changeset.deletes {
            events.push(Event::TreeVanished { id: id.into() })
        }

        let next_id = *[self.id, self.trees.last_id()].iter().max().unwrap();
        if next_id != self.id {
            info!("Advance id value from {} to {}", self.id, next_id);
            self.id = next_id;
        }

        events
    }
}
