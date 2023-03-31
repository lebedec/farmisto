use crate::collections::Shared;
use crate::inventory::Inventory::ItemAdded;
use crate::inventory::{
    ContainerId, Function, Inventory, InventoryDomain, InventoryError, Item, ItemId, ItemKind,
};

impl InventoryDomain {
    pub fn create_item<'operation>(
        &'operation mut self,
        kind: &Shared<ItemKind>,
        container: ContainerId,
        quantity: u8,
    ) -> Result<(ItemId, impl FnOnce() -> Vec<Inventory> + 'operation), InventoryError> {
        self.get_container(container)?; // ensure container valid
        let id = ItemId(self.items_sequence + 1);
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
            self.items_sequence += 1;
            let container = self.get_mut_container(container).unwrap();
            container.items.push(item);
            events
        };
        Ok((id, operation))
    }
}
