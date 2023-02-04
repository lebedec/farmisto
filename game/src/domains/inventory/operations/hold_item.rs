use std::collections::HashMap;

use crate::collections::Shared;
use crate::inventory::Inventory::{ContainerCreated, ItemAdded, ItemRemoved};
use crate::inventory::InventoryError::{ContainerNotFound, ItemNotFound};
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError, Item, ItemId,
};

pub struct ItemHold<'operation> {
    source: ContainerId,
    pub container: Container,
    item_index: usize,
    containers: &'operation mut Vec<Container>,
    containers_sequence: &'operation mut usize,
    items: &'operation mut HashMap<ContainerId, Vec<Item>>,
}

impl<'operation> ItemHold<'operation> {
    pub fn complete(self) -> Vec<Inventory> {
        let mut item = self
            .items
            .get_mut(&self.source)
            .unwrap()
            .remove(self.item_index);
        item.container = self.container.id;
        let events = vec![
            ItemRemoved {
                item: item.id,
                container: self.source,
            },
            ContainerCreated {
                id: self.container.id,
            },
            ItemAdded {
                item: item.id,
                kind: item.kind.id,
                container: item.container,
            },
        ];
        let items = self.items.entry(self.container.id).or_insert(vec![]);
        items.push(item);
        *self.containers_sequence = self.container.id.0;
        self.containers.push(self.container);
        events
    }
}

impl InventoryDomain {
    pub fn hold_item(
        &mut self,
        container: ContainerId,
        item: ItemId,
        kind: Shared<ContainerKind>,
    ) -> Result<ItemHold, InventoryError> {
        if !self.items.contains_key(&container) {
            return Err(ContainerNotFound { container });
        }
        let item_index = self
            .items
            .get(&container)
            .unwrap()
            .iter()
            .position(|search| search.id == item)
            .ok_or(ItemNotFound { item, container })?;
        let hold = ItemHold {
            source: container,
            container: Container {
                id: ContainerId(self.containers_sequence + 1),
                kind,
            },
            item_index,
            containers: &mut self.containers,
            containers_sequence: &mut self.containers_sequence,
            items: &mut self.items,
        };
        Ok(hold)
    }
}
