use crate::inventory::domain::InventoryDomain;
use crate::inventory::{ContainerId, Inventory, InventoryError, Item};

impl InventoryDomain {
    pub fn use_items_from<'operation>(
        &'operation mut self,
        container: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        let container = self.get_mut_container(container)?;
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
