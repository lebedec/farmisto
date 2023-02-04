use crate::collections::Shared;
use std::collections::hash_map::Values;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerKey(pub usize);

pub struct ContainerKind {
    pub id: ContainerKey,
    pub name: String,
    pub capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct ContainerId(pub usize);

pub struct Container {
    pub id: ContainerId,
    pub kind: Shared<ContainerKind>,
}

#[derive(Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum Function {
    Material { keyword: String },
    Carry,
    Hammer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct ItemKey(pub usize);

pub struct ItemKind {
    pub id: ItemKey,
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct ItemId(pub(crate) usize);

pub struct Item {
    pub id: ItemId,
    pub kind: Shared<ItemKind>,
    pub container: ContainerId,
}

#[derive(bincode::Encode, bincode::Decode)]
pub enum Inventory {
    ContainerCreated {
        id: ContainerId,
    },
    ContainerDestroyed {
        id: ContainerId,
    },
    ItemAdded {
        item: ItemId,
        kind: ItemKey,
        container: ContainerId,
    },
    ItemRemoved {
        item: ItemId,
        container: ContainerId,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum InventoryError {
    ContainerNotFound {
        container: ContainerId,
    },
    ContainerEmpty {
        container: ContainerId,
    },
    ItemNotFound {
        container: ContainerId,
        item: ItemId,
    },
}

#[derive(Default)]
pub struct InventoryDomain {
    pub known_containers: HashMap<ContainerKey, Shared<ContainerKind>>,
    pub known_items: HashMap<ItemKey, Shared<ItemKind>>,
    pub items: HashMap<ContainerId, Vec<Item>>,
    pub items_sequence: usize,
    pub containers: Vec<Container>,
    pub containers_sequence: usize,
}

impl InventoryDomain {
    pub fn load_containers(&mut self, containers: Vec<Container>, sequence: usize) {
        self.containers_sequence = sequence;
        self.containers.extend(containers);
    }

    pub fn load_items(&mut self, items: Vec<Item>, sequence: usize) {
        self.items_sequence = sequence;
        for item in items {
            let items = self.items.entry(item.container).or_insert(vec![]);
            items.push(item);
        }
    }

    pub fn get_all_items(&self) -> Values<ContainerId, Vec<Item>> {
        self.items.values()
    }

    pub fn get_items(&self, container: ContainerId) -> Result<&Vec<Item>, InventoryError> {
        match self.items.get(&container) {
            None => Err(InventoryError::ContainerEmpty { container }),
            Some(items) => Ok(items),
        }
    }
}
