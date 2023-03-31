use crate::inventory::domain::InventoryDomain;
use crate::inventory::Inventory::ItemQuantityChanged;
use crate::inventory::InventoryError::ItemQuantityOverflow;
use crate::inventory::{ContainerId, Inventory, InventoryError};

impl InventoryDomain {
    pub fn increase_item<'operation>(
        &'operation mut self,
        container: ContainerId,
        increment: u8,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + 'operation, InventoryError> {
        let container = self.get_mut_container(container)?;
        let index = container.ensure_item_at(-1)?;
        if container.items[index].quantity > u8::MAX - increment {
            return Err(ItemQuantityOverflow {
                id: container.items[index].id,
            });
        }
        let operation = move || {
            let mut events = vec![];
            let item = &mut container.items[index];
            item.quantity += increment;
            events.push(ItemQuantityChanged {
                id: item.id,
                container: item.container,
                quantity: item.quantity,
            });
            events
        };
        Ok(operation)
    }
}
