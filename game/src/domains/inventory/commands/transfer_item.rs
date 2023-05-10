use crate::inventory::Inventory::{ContainerDestroyed, ItemRemoved, ItemsAdded};

use crate::inventory::InventoryDomain;
use crate::inventory::InventoryError;
use crate::inventory::{ContainerId, Function};
use crate::inventory::{Inventory, ItemData};
use crate::math::{Position, VectorMath};
use crate::physics::SpaceId;

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

    pub fn validate_items_filter(filter: &Vec<Function>, functions: &Vec<Function>) -> bool {
        if filter.is_empty() {
            true
        } else {
            for function in functions {
                if filter.contains(function) {
                    return true;
                }
            }
            false
        }
    }

    pub fn transfer_item<'operation>(
        &'operation mut self,
        source: ContainerId,
        offset: isize,
        destination_id: ContainerId,
        keep_container: bool,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        let source_container = self.get_container(source)?;
        let index = source_container.ensure_item_at(offset)?;
        let item = &source_container.items[index];
        let item_functions = item.kind.functions.clone();
        let item_id = item.id;
        let keep_container = keep_container || source_container.items.len() > 1;
        let destination = self.get_mut_container(destination_id)?;

        // validations:
        if let Some(item_on_top) = destination.items.last() {
            if !item_on_top.kind.stackable {
                return Err(InventoryError::NonStackableItemOnTop {
                    container: destination.id,
                    item: item_on_top.id,
                });
            }
        }
        if !Self::validate_items_filter(&destination.kind.filter, &item_functions) {
            return Err(InventoryError::ContainerFilterError {
                container: destination.id,
                item: item_id,
            });
        }
        if destination.items.len() + 1 > destination.kind.capacity {
            return Err(InventoryError::ContainerIsFull { id: destination.id });
        }

        let command = move || {
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
            let destination = self.containers.get_mut(&destination_id).unwrap();
            item.container = destination.id;
            events.push(ItemsAdded {
                items: vec![ItemData {
                    id: item.id,
                    key: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                }],
            });
            destination.items.push(item);
            events
        };
        Ok(command)
    }
}
