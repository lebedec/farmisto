use crate::inventory::Inventory::{ContainerDestroyed, ItemRemoved};
use crate::inventory::InventoryError::ContainerHasItems;
use crate::inventory::{ContainerId, Inventory, InventoryDomain, InventoryError};

impl InventoryDomain {
    pub fn destroy_containers<'operation>(
        &'operation mut self,
        containers: Vec<ContainerId>,
        force: bool,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        for id in &containers {
            let container = self.get_container(*id)?;
            if !force && container.items.len() > 0 {
                return Err(ContainerHasItems { id: *id });
            }
        }
        let operation = || {
            let mut events = vec![];
            for id in containers {
                let container = self.containers.remove(&id).unwrap();
                for item in container.items {
                    events.push(ItemRemoved {
                        item: item.id,
                        container: item.container,
                    })
                }
                events.push(ContainerDestroyed { id });
            }
            events
        };
        Ok(operation)
    }
}
