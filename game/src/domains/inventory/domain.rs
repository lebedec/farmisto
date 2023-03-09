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

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
pub enum Function {
    Material { keyword: usize },
    Installation { kind: usize },
    Seeding { kind: usize },
    Carry,
    Instrumenting,
    Shovel,
    Product { kind: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct ItemKey(pub usize);

pub struct ItemKind {
    pub id: ItemKey,
    pub name: String,
    pub stackable: Option<u8>,
    pub max_quantity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct ItemId(pub usize);

pub struct Item {
    pub id: ItemId,
    pub kind: Shared<ItemKind>,
    pub container: ContainerId,
    pub functions: Vec<Function>,
    pub quantity: u8,
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
        id: ItemId,
        kind: ItemKey,
        functions: Vec<Function>,
        container: ContainerId,
        quantity: u8,
    },
    ItemRemoved {
        item: ItemId,
        container: ContainerId,
    },
    ItemQuantityChanged {
        id: ItemId,
        container: ContainerId,
        quantity: u8,
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
    ItemFunctionNotFound {
        id: ItemId,
    },
    ItemQuantityOverflow {
        id: ItemId,
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

    pub fn get_container_item(&self, id: ContainerId) -> Result<&Item, InventoryError> {
        let container = self.get_container(id)?;
        container.items.get(0).ok_or(ItemNotFoundByIndex {
            container: id,
            index: 0,
        })
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

impl Item {
    pub fn as_seeds(&self) -> Result<usize, InventoryError> {
        for function in &self.functions {
            if let Function::Seeding { kind } = function {
                return Ok(*kind);
            }
        }
        Err(InventoryError::ItemFunctionNotFound { id: self.id })
    }

    pub fn as_hammer(&self) -> Result<(), InventoryError> {
        for function in &self.functions {
            if let Function::Instrumenting = function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound { id: self.id })
    }

    pub fn as_shovel(&self) -> Result<(), InventoryError> {
        for function in &self.functions {
            if let Function::Shovel = function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound { id: self.id })
    }

    pub fn as_product(&self) -> Result<usize, InventoryError> {
        for function in &self.functions {
            if let Function::Product { kind } = function {
                return Ok(*kind);
            }
        }
        Err(InventoryError::ItemFunctionNotFound { id: self.id })
    }

    pub fn as_installation(&self) -> Result<usize, InventoryError> {
        for function in &self.functions {
            if let Function::Installation { kind } = function {
                return Ok(*kind);
            }
        }
        Err(InventoryError::ItemFunctionNotFound { id: self.id })
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
