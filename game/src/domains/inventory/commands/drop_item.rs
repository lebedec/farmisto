use crate::collections::{trust};
use crate::inventory::Inventory::{ContainerCreated, ItemRemoved, ItemsAdded};
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError, ItemData,
};
use crate::timing::Shared;

impl InventoryDomain {
    pub fn drop_item(
        &mut self,
        source_id: ContainerId,
        destination_kind: Shared<ContainerKind>,
        destination_id: ContainerId,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + '_, InventoryError> {
        let source = self.mut_container(source_id)?;
        let index = source.ensure_item_at(-1)?;
        let mut source = trust(source);
        let command = move || {
            let mut item = source.items.remove(index);
            item.container = destination_id;
            let data = ItemData {
                id: item.id,
                key: item.kind.id,
                container: item.container,
                quantity: item.quantity,
            };
            self.containers.insert(
                destination_id,
                Container {
                    id: destination_id,
                    kind: destination_kind.clone(),
                    items: vec![item],
                },
            );
            self.containers_id.register(destination_id.0);
            vec![
                ItemRemoved {
                    item: data.id,
                    container: source_id,
                },
                ContainerCreated { id: destination_id },
                ItemsAdded { items: vec![data] },
            ]
        };
        Ok(command)
    }
}
