use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::collections::{Sequence, Shared};
use crate::inventory::InventoryError::{ContainerNotFound, ItemNotFound, ItemNotFoundByIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerKey(pub usize);

pub struct ContainerKind {
    pub id: ContainerKey,
    pub name: String,
    pub capacity: usize,
    pub filter: Vec<Function>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(pub usize);

pub struct Container {
    pub id: ContainerId,
    pub kind: Shared<ContainerKind>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub struct Nozzle {
    pub pressure: f32,
    pub spread: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum Function {
    Material(u8),
    Installation(usize),
    Seeding(usize),
    Carry,
    Instrumenting,
    Shovel,
    Product(usize),
    Assembly(usize),
    Stone,
    Moistener(Nozzle),
    Fertilizer(f32),
    Residue,
    Food,
    Tether,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemKey(pub usize);

pub struct ItemKind {
    pub id: ItemKey,
    pub name: String,
    pub stackable: bool,
    pub functions: Vec<Function>,
    pub max_quantity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub usize);

pub struct Item {
    pub id: ItemId,
    pub kind: Shared<ItemKind>,
    pub container: ContainerId,
    pub quantity: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemData {
    pub id: ItemId,
    pub key: ItemKey,
    pub container: ContainerId,
    pub quantity: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Inventory {
    ContainerCreated {
        id: ContainerId,
    },
    ContainerDestroyed {
        id: ContainerId,
    },
    ItemsAdded {
        items: Vec<ItemData>,
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

#[derive(Debug, Serialize, Deserialize)]
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
    ItemFunctionNotFound,
    ItemQuantityOverflow {
        id: ItemId,
    },
    ContainerFilterError {
        container: ContainerId,
        item: ItemId,
    },
    NonStackableItemOnTop {
        container: ContainerId,
        item: ItemId,
    },
}

#[derive(Default)]
pub struct InventoryDomain {
    pub items_id: Sequence,
    pub containers: HashMap<ContainerId, Container>,
    pub containers_id: Sequence,
}

impl InventoryDomain {
    pub fn load_containers(&mut self, containers: Vec<Container>, sequence: usize) {
        self.containers_id.set(sequence);
        for container in containers {
            self.containers.insert(container.id, container);
        }
    }

    pub fn load_items(&mut self, items: Vec<Item>, sequence: usize) {
        self.items_id.set(sequence);
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

    pub fn get_container_item_for_use(
        &self,
        id: ContainerId,
    ) -> Result<(bool, &Item), InventoryError> {
        let container = self.get_container(id)?;
        let is_last = container.items.len() == 1;
        container
            .items
            .get(0)
            .map(|item| (is_last, item))
            .ok_or(ItemNotFoundByIndex {
                container: id,
                index: 0,
            })
    }

    pub fn mut_container(&mut self, id: ContainerId) -> Result<&mut Container, InventoryError> {
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
