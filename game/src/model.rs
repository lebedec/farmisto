use crate::persistence::{Collection, Id, Knowledge, Persisted, Shared};
use crate::physics::BarrierId;
use crate::planting::PlantId;

#[derive(Default)]
pub struct Universe {
    known_trees: Knowledge<TreeKind>,
    trees: Collection<Tree>,
}

#[derive(Persisted)]
pub struct TreeKind {
    pub id: usize,
    pub name: String,
    pub barrier: usize,
    pub plant: usize,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeId(usize);

#[derive(Persisted)]
pub struct Tree {
    pub id: TreeId,
    pub kind: Shared<TreeKind>,
    pub barrier: BarrierId,
    pub plant: PlantId,
}

impl Universe {
    pub fn load(&mut self, connection: &rusqlite::Connection) {
        self.known_trees.load(connection);
        self.trees.load(connection, &self.known_trees);
    }
}
