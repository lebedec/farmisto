use std::collections::HashMap;

use crate::inventory::domain::InventoryDomain;
use crate::inventory::{ContainerId, Inventory, InventoryError, Item};

pub struct ItemsUsing<'operation> {
    container: ContainerId,
    items: &'operation mut HashMap<ContainerId, Vec<Item>>,
}

impl<'operation> ItemsUsing<'operation> {
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

impl InventoryDomain {
    pub fn use_items_from(&mut self, container: ContainerId) -> Result<ItemsUsing, InventoryError> {
        if self.items.contains_key(&container) {
            Ok(ItemsUsing {
                container,
                items: &mut self.items,
            })
        } else {
            Err(InventoryError::ContainerNotFound { container })
        }
    }
}
