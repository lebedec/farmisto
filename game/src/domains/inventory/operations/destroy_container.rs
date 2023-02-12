use crate::inventory::Inventory::{ContainerDestroyed, ItemRemoved};
use crate::inventory::InventoryError::ContainerHasItems;
use crate::inventory::{ContainerId, Inventory, InventoryDomain, InventoryError};

impl InventoryDomain {
    pub fn destroy_container<'operation>(
        &'operation mut self,
        id: ContainerId,
        force: bool,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        let container = self.get_container(id)?;
        if !force && container.items.len() > 0 {
            return Err(ContainerHasItems { id });
        }
        let operation = move || {
            let container = self.containers.remove(&id).unwrap();
            let mut events = vec![];
            for item in container.items {
                events.push(ItemRemoved {
                    item: item.id,
                    container: item.container,
                })
            }
            events.push(ContainerDestroyed { id });
            events
        };
        Ok(operation)
    }
}
