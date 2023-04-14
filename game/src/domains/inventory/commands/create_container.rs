use crate::collections::{Shared, TemporaryRef};
use crate::inventory::Inventory::{ContainerCreated, ItemAdded};
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError, Item,
};

impl InventoryDomain {
    pub fn add_container(
        &mut self,
        id: ContainerId,
        kind: &Shared<ContainerKind>,
        items: Vec<Item>,
    ) -> Result<impl FnOnce() -> Vec<Inventory>, InventoryError> {
        let mut container = Container {
            id,
            kind: kind.clone(),
            items: vec![],
        };
        let mut domain = TemporaryRef::from(self);
        let command = move || {
            let mut events = vec![ContainerCreated { id }];
            for item in items {
                events.push(ItemAdded {
                    id: item.id,
                    kind: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                });
                domain.items_id.register(item.id.0);
                container.items.push(item);
            }
            domain.containers_id.register(id.0);
            domain.containers.insert(id, container);
            events
        };
        Ok(command)
    }

    pub fn add_empty_container(
        &mut self,
        id: ContainerId,
        kind: &Shared<ContainerKind>,
    ) -> Result<impl FnOnce() -> Vec<Inventory>, InventoryError> {
        self.add_container(id, kind, vec![])
    }
}
