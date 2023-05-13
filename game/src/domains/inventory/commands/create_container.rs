use crate::collections::{trust, Shared};
use crate::inventory::Inventory::{ContainerCreated, ItemsAdded};
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError, Item,
    ItemData,
};

impl InventoryDomain {
    pub fn add_container(
        &mut self,
        id: ContainerId,
        kind: &Shared<ContainerKind>,
        container_items: Vec<Item>,
    ) -> Result<impl FnOnce() -> Vec<Inventory>, InventoryError> {
        let mut container = Container {
            id,
            kind: kind.clone(),
            items: vec![],
        };
        let mut domain = trust(self);
        let command = move || {
            let mut items = Vec::with_capacity(container_items.len());
            for item in container_items {
                items.push(ItemData {
                    id: item.id,
                    key: item.kind.id,
                    container: item.container,
                    quantity: item.quantity,
                });
                domain.items_id.register(item.id.0);
                container.items.push(item);
            }
            let events = vec![ContainerCreated { id }, ItemsAdded { items }];
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
