use crate::collections::Shared;
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
    pub cell: [f32; 2],
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Function {
    Material { keyword: String },
    Carry,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
        container: ContainerId,
    },
    ItemRemoved {
        item: ItemId,
        container: ContainerId,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum InventoryError {
    ContainerNotFound { container: ContainerId },
}

#[derive(Default)]
pub struct InventoryDomain {
    pub known_containers: HashMap<ContainerKey, Shared<ContainerKind>>,
    items: HashMap<ContainerId, Vec<Item>>,
    containers: Vec<Container>,
}

pub struct Usage<'action> {
    container: ContainerId,
    items: &'action mut HashMap<ContainerId, Vec<Item>>,
}

impl<'action> Usage<'action> {
    pub fn items(&self) -> &Vec<Item> {
        self.items.get(&self.container).unwrap()
    }

    pub fn complete(self) -> Vec<Inventory> {
        let mut events = vec![];
        let items = self.items.remove(&self.container).unwrap();
        for item in items {
            events.push(Inventory::ItemRemoved {
                item: item.id,
                container: self.container,
            })
        }
        events
    }
}

pub struct ContainerCreation<'action> {
    pub container: Container,
    containers: &'action mut Vec<Container>,
}

impl<'action> ContainerCreation<'action> {
    pub fn complete(self) -> Vec<Inventory> {
        let events = vec![Inventory::ContainerCreated {
            id: self.container.id,
        }];
        self.containers.push(self.container);
        events
    }
}

impl InventoryDomain {
    pub fn get_container(&self, id: ContainerId) -> Option<&Container> {
        self.containers.iter().find(|container| container.id == id)
    }

    pub fn create_container(
        &mut self,
        kind: Shared<ContainerKind>,
    ) -> Result<ContainerCreation, InventoryError> {
        Ok(ContainerCreation {
            container: Container {
                id: ContainerId(self.containers.len()),
                kind,
                cell: [0.0, 0.0],
            },
            containers: &mut self.containers,
        })
    }

    pub fn use_items_from(&mut self, container: ContainerId) -> Result<Usage, InventoryError> {
        if self.items.contains_key(&container) {
            Ok(Usage {
                container,
                items: &mut self.items,
            })
        } else {
            Err(InventoryError::ContainerNotFound { container })
        }
    }

    pub fn index_container(&self, id: ContainerId) -> Option<usize> {
        self.containers
            .iter()
            .position(|container| container.id == id)
    }

    pub fn drop_item(&mut self, item: ItemId, cell: [usize; 2]) {}

    pub fn put_item(&mut self, container: ContainerId, item: ItemId) {}

    pub fn pop_item(&mut self, container: ContainerId, item: ItemId) {}

    pub fn get_items(&self, container: ContainerId) -> Option<&Vec<Item>> {
        self.items.get(&container)
    }
}
