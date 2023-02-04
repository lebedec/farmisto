use std::collections::HashMap;

use crate::inventory::Inventory::{ItemAdded, ItemRemoved};
use crate::inventory::{ContainerId, Inventory, InventoryDomain, InventoryError, Item, ItemId};

pub struct ItemTransfer<'operation> {
    items: &'operation mut HashMap<ContainerId, Vec<Item>>,
    source: ContainerId,
    destination: ContainerId,
    item_index: usize,
}

impl<'operation> ItemTransfer<'operation> {
    pub fn complete(self) -> Vec<Inventory> {
        let mut item = self
            .items
            .get_mut(&self.source)
            .unwrap()
            .remove(self.item_index);
        item.container = self.destination;
        let events = vec![
            ItemRemoved {
                item: item.id,
                container: self.source,
            },
            ItemAdded {
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
    ) -> Result<ItemTransfer, InventoryError> {
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
        let transfer = ItemTransfer {
            items: &mut self.items,
            source,
            destination,
            item_index,
        };
        Ok(transfer)
    }
}
