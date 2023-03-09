use crate::inventory::domain::InventoryDomain;
use crate::inventory::Inventory::{ItemQuantityChanged, ItemRemoved};
use crate::inventory::{ContainerId, Inventory, InventoryError, Item};

impl InventoryDomain {
    pub fn decrease_item<'operation>(
        &'operation mut self,
        container: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        let container = self.get_mut_container(container)?;
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
