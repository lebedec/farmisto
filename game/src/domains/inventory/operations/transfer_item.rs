use crate::inventory::Inventory::{ContainerDestroyed, ItemAdded, ItemRemoved};

use crate::inventory::ContainerId;
use crate::inventory::Inventory;
use crate::inventory::InventoryDomain;
use crate::inventory::InventoryError;

impl InventoryDomain {
    pub fn pop_item_and_destroy<'operation>(
        &'operation mut self,
        source: ContainerId,
        destination: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        self.transfer_item(source, -1, destination, false)
    }

    pub fn pop_item<'operation>(
        &'operation mut self,
        source: ContainerId,
        destination: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        self.transfer_item(source, -1, destination, true)
    }

    pub fn transfer_item<'operation>(
        &'operation mut self,
        source: ContainerId,
        offset: isize,
        destination: ContainerId,
        keep_container: bool,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        // SAFETY: self borrowing and closure guarantees safe ptr handling
        let source_container = self.get_container(source)?;
        let index = source_container.ensure_item_at(offset)?;
        let keep_container = keep_container || source_container.items.len() > 1;
        let destination_container = self.get_mut_container(destination)?;
        if destination_container.items.len() + 1 > destination_container.kind.capacity {
            return Err(InventoryError::ContainerIsFull {
                id: destination_container.id,
            });
        }
        let operation = move || {
            let mut events = vec![];
            let mut item = if !keep_container {
                let mut container = self.containers.remove(&source).unwrap();
                let item = container.items.remove(index);
                events.extend([
                    ItemRemoved {
                        item: item.id,
                        container: container.id,
                    },
                    ContainerDestroyed { id: container.id },
                ]);
                item
            } else {
                let container = self.containers.get_mut(&source).unwrap();
                let item = container.items.remove(index);
                events.extend([ItemRemoved {
                    item: item.id,
                    container: container.id,
                }]);
                item
            };
            let destination = self.containers.get_mut(&destination).unwrap();
            item.container = destination.id;
            events.push(ItemAdded {
                item: item.id,
                kind: item.kind.id,
                container: item.container,
            });
            destination.items.push(item);
            events
        };
        Ok(operation)
    }
}
