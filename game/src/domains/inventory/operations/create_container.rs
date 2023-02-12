use crate::collections::Shared;
use crate::inventory::Inventory::ContainerCreated;
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError,
};

impl InventoryDomain {
    pub fn create_container<'operation>(
        &'operation mut self,
        kind: Shared<ContainerKind>,
    ) -> Result<(ContainerId, impl FnOnce() -> Vec<Inventory> + 'operation), InventoryError> {
        let id = ContainerId(self.containers_sequence + 1);
        let container = Container {
            id,
            kind,
            items: vec![],
        };
        let operation = move || {
            self.containers_sequence += 1;
            self.containers.insert(id, container);
            vec![ContainerCreated { id }]
        };
        Ok((id, operation))
    }
}
