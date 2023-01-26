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
}

#[derive(Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum Function {
    Material { keyword: String },
    Carry,
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
    ItemNotFound {
        container: ContainerId,
        item: ItemId,
    },
}

#[derive(Default)]
pub struct InventoryDomain {
    pub known_containers: HashMap<ContainerKey, Shared<ContainerKind>>,
    pub known_items: HashMap<ItemKey, Shared<ItemKind>>,
    pub(crate) items: HashMap<ContainerId, Vec<Item>>,
    pub(crate) containers: Vec<Container>,
}

pub struct Transfer<'action> {
    items: &'action mut HashMap<ContainerId, Vec<Item>>,
    source: ContainerId,
    destination: ContainerId,
    item_index: usize,
}

impl<'action> Transfer<'action> {
    pub fn complete(self) -> Vec<Inventory> {
        let mut item = self
            .items
            .get_mut(&self.source)
            .unwrap()
            .remove(self.item_index);
        item.container = self.destination;
        let events = vec![
            Inventory::ItemRemoved {
                item: item.id,
                container: self.source,
            },
            Inventory::ItemAdded {
                item: item.id,
                kind: item.kind.id,
                container: item.container,
            },
        ];
        let items = self.items.entry(self.destination).or_insert(vec![]);
        items.push(item);
        events
    }
}

impl InventoryDomain {
    pub(crate) fn transfer_item(
        &mut self,
        source: ContainerId,
        item: ItemId,
        destination: ContainerId,
    ) -> Result<Transfer, InventoryError> {
        if !self.items.contains_key(&source) {
            return Err(InventoryError::ContainerNotFound { container: source });
        }
        // destroy
        // if !self.items.contains_key(&destination) {
        //     return Err(InventoryError::ContainerNotFound {
        //         container: destination,
        //     });
        // }
        // capacity
        let item_index = self
            .items
            .get(&source)
            .unwrap()
            .iter()
            .position(|search| search.id == item)
            .ok_or(InventoryError::ItemNotFound {
                item,
                container: source,
            })?;
        let transfer = Transfer {
            items: &mut self.items,
            source,
            destination,
            item_index,
        };
        Ok(transfer)
    }
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

pub struct ItemHold<'action> {
    source: ContainerId,
    pub container: Container,
    item_index: usize,
    containers: &'action mut Vec<Container>,
    items: &'action mut HashMap<ContainerId, Vec<Item>>,
}

impl InventoryDomain {
    pub fn hold_item(
        &mut self,
        source: ContainerId,
        item: ItemId,
        kind: Shared<ContainerKind>,
    ) -> Result<ItemHold, InventoryError> {
        if !self.items.contains_key(&source) {
            return Err(InventoryError::ContainerNotFound { container: source });
        }
        let item_index = self
            .items
            .get(&source)
            .unwrap()
            .iter()
            .position(|search| search.id == item)
            .ok_or(InventoryError::ItemNotFound {
                item,
                container: source,
            })?;
        let hold = ItemHold {
            source,
            container: Container {
                id: ContainerId(self.containers.len() + 1),
                kind,
            },
            item_index,
            containers: &mut self.containers,
            items: &mut self.items,
        };
        Ok(hold)
    }
}

impl<'action> ItemHold<'action> {
    pub fn complete(self) -> Vec<Inventory> {
        let mut item = self
            .items
            .get_mut(&self.source)
            .unwrap()
            .remove(self.item_index);
        item.container = self.container.id;
        let events = vec![
            Inventory::ItemRemoved {
                item: item.id,
                container: self.source,
            },
            Inventory::ContainerCreated {
                id: self.container.id,
            },
            Inventory::ItemAdded {
                item: item.id,
                kind: item.kind.id,
                container: item.container,
            },
        ];
        let items = self.items.entry(self.container.id).or_insert(vec![]);
        items.push(item);
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
                id: ContainerId(self.containers.len() + 1),
                kind,
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
