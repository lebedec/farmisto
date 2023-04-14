use crate::collections::Shared;
use crate::inventory::Inventory::ItemAdded;
use crate::inventory::{
    ContainerId, Inventory, InventoryDomain, InventoryError, Item, ItemId, ItemKind,
};

impl InventoryDomain {
    pub fn create_item(
        &mut self,
        id: ItemId,
        kind: &Shared<ItemKind>,
        container: ContainerId,
        quantity: u8,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + '_, InventoryError> {
        self.get_container(container)?; // ensure container valid
        let item = Item {
            id,
            kind: kind.clone(),
            container,
            quantity,
        };
        let operation = move || {
            let events = vec![ItemAdded {
                id: item.id,
                kind: item.kind.id,
                container,
                quantity: item.quantity,
            }];
            self.items_id.register(item.id.0);
            let container = self.get_mut_container(container).unwrap();
            container.items.push(item);
            events
        };
        Ok(operation)
    }
}
