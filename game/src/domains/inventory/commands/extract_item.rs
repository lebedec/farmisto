use crate::collections::Shared;
use crate::inventory::Inventory::{ContainerCreated, ItemRemoved, ItemsAdded};
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError, ItemData,
};

impl InventoryDomain {
    pub fn extract_item(
        &mut self,
        source: ContainerId,
        offset: isize,
        id: ContainerId,
        kind: Shared<ContainerKind>,
    ) -> Result<impl FnOnce() -> Vec<Inventory> + '_, InventoryError> {
        let source_container = self.get_mut_container(source)?;
        let index = source_container.ensure_item_at(offset)?;
        let mut container = Container {
            id,
            kind,
            items: vec![],
        };
        let commit = move || {
            let source = self.get_mut_container(source).unwrap();
            let mut item = source.items.remove(index);
            item.container = container.id;
            let events = vec![
                ItemRemoved {
                    item: item.id,
                    container: source.id,
                },
                ContainerCreated { id: container.id },
                ItemsAdded {
                    items: vec![ItemData {
                        id: item.id,
                        key: item.kind.id,
                        container: item.container,
                        quantity: item.quantity,
                    }],
                },
            ];
            container.items.push(item);
            self.containers_id.register(id.0);
            self.containers.insert(container.id, container);
            events
        };
        Ok(commit)
    }
}
