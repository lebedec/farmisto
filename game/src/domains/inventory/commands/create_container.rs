use crate::collections::{Shared, TemporaryRef};
use crate::inventory::Inventory::ContainerCreated;
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError,
};

impl InventoryDomain {
    pub fn add_container(
        &mut self,
        id: ContainerId,
        kind: &Shared<ContainerKind>,
    ) -> Result<impl FnOnce() -> Vec<Inventory>, InventoryError> {
        let container = Container {
            id,
            kind: kind.clone(),
            items: vec![],
        };
        let mut domain = TemporaryRef::from(self);
        let command = move || {
            domain.containers_id.register(id.0);
            domain.containers.insert(id, container);
            vec![ContainerCreated { id }]
        };
        Ok(command)
    }
}
