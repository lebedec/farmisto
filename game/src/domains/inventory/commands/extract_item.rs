use crate::collections::Shared;
use crate::inventory::Inventory::{ContainerCreated, ItemAdded, ItemRemoved};
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError, ItemId,
};

impl InventoryDomain {
    pub fn extract_item<'operation>(
        &'operation mut self,
        source: ContainerId,
        offset: isize,
        kind: Shared<ContainerKind>,
    ) -> Result<(ContainerId, impl FnOnce() -> Vec<Inventory> + 'operation), InventoryError> {
        let source_container = self.get_mut_container(source)?;
        let index = source_container.ensure_item_at(offset)?;
        let id = ContainerId(self.containers_sequence + 1);
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
                ItemAdded {
                    id: item.id,
                    kind: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                },
            ];
            container.items.push(item);
            self.containers_sequence += 1;
            self.containers.insert(container.id, container);
            events
        };
        Ok((id, commit))
    }
}
