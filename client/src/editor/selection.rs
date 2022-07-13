use game::model::{FarmlandId, TreeId};

#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    FarmlandProp {
        id: usize,
        farmland: FarmlandId,
        kind: String,
    },
    Tree {
        id: TreeId,
    },
}
