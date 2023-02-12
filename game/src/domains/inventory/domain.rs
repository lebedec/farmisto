use crate::collections::Shared;
use crate::inventory::InventoryError::{
    ContainerNotFound, ContainersNotFound, ItemNotFound, ItemNotFoundByIndex,
};
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
    pub items: Vec<Item>,
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
pub struct ItemId(pub usize);

pub struct Item {
    pub id: ItemId,
    pub kind: Shared<ItemKind>,
    pub container: ContainerId,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
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
    ContainersNotFound,
    ContainerNotFound {
        id: ContainerId,
    },
    ContainerIsFull {
        id: ContainerId,
    },
    ContainerHasItems {
        id: ContainerId,
    },
    ItemNotFound {
        container: ContainerId,
        item: ItemId,
    },
    ItemNotFoundByIndex {
        container: ContainerId,
        index: isize,
    },
}

#[derive(Default)]
pub struct InventoryDomain {
    pub items_sequence: usize,
    pub containers: HashMap<ContainerId, Container>,
    pub containers_sequence: usize,
}

impl InventoryDomain {
    pub fn load_containers(&mut self, containers: Vec<Container>, sequence: usize) {
        self.containers_sequence = sequence;
        for container in containers {
            self.containers.insert(container.id, container);
        }
    }

    pub fn load_items(&mut self, items: Vec<Item>, sequence: usize) {
        self.items_sequence = sequence;
        for item in items {
            let container = self.containers.get_mut(&item.container).unwrap();
            container.items.push(item);
        }
    }

    pub fn get_container(&self, id: ContainerId) -> Result<&Container, InventoryError> {
        self.containers.get(&id).ok_or(ContainerNotFound { id })
    }

    pub fn get_mut_container(&mut self, id: ContainerId) -> Result<&mut Container, InventoryError> {
        self.containers.get_mut(&id).ok_or(ContainerNotFound { id })
    }

    pub fn get_mut_container_ptr(
        &mut self,
        id: ContainerId,
    ) -> Result<*mut Container, InventoryError> {
        let container = self
            .containers
            .get_mut(&id)
            .ok_or(ContainerNotFound { id })?;
        Ok(container)
    }
}

impl Container {
    pub fn get_item(&self, id: ItemId) -> Result<&Item, InventoryError> {
        self.items
            .iter()
            .find(|item| item.id == id)
            .ok_or(ItemNotFound {
                container: self.id,
                item: id,
            })
    }

    pub fn ensure_item_at(&self, offset: isize) -> Result<usize, InventoryError> {
        let index = if offset < 0 {
            self.items.len() as isize + offset
        } else {
            offset
        } as usize;
        if index < self.items.len() {
            Ok(index)
        } else {
            Err(ItemNotFoundByIndex {
                container: self.id,
                index: offset,
            })
        }
    }

    pub fn index_item(&self, id: ItemId) -> Result<usize, InventoryError> {
        self.items
            .iter()
            .position(|item| item.id == id)
            .ok_or(ItemNotFound {
                container: self.id,
                item: id,
            })
    }
}
