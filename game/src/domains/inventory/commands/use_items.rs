use crate::inventory::domain::InventoryDomain;
use crate::inventory::{ContainerId, Inventory, InventoryError};

impl InventoryDomain {
    pub fn pop_use_item(
        &mut self,
        container: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + '_, InventoryError> {
        let container = self.mut_container(container)?;
        let operation = move || {
            let mut events = vec![];
            let item = container.items.remove(0);
            events.push(Inventory::ItemRemoved {
                item: item.id,
                container: item.container,
            });
            events
        };
        Ok(operation)
    }

    pub fn use_items_from(
        &mut self,
        container: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + '_, InventoryError> {
        let container = self.mut_container(container)?;
        let operation = move || {
            let mut events = vec![];
            for item in &container.items {
                events.push(Inventory::ItemRemoved {
                    item: item.id,
                    container: item.container,
                })
            }
            container.items.clear();
            events
        };
        Ok(operation)
    }
}
