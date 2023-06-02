use crate::inventory::domain::InventoryDomain;
use crate::inventory::Inventory::{ItemQuantityChanged, ItemRemoved};
use crate::inventory::{ContainerId, Inventory, InventoryError};

impl InventoryDomain {
    pub fn decrease_container_item(
        &mut self,
        container: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + '_, InventoryError> {
        let container = self.mut_container(container)?;
        let index = container.ensure_item_at(-1)?;
        let operation = move || {
            let mut events = vec![];
            if container.items[index].quantity > 1 {
                let item = &mut container.items[index];
                item.quantity -= 1;
                events.push(ItemQuantityChanged {
                    id: item.id,
                    container: item.container,
                    quantity: item.quantity,
                })
            } else {
                let item = container.items.remove(index);
                events.push(ItemRemoved {
                    item: item.id,
                    container: item.container,
                })
            }
            events
        };
        Ok(operation)
    }
}
