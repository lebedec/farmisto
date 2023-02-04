use crate::collections::Shared;
use crate::inventory::Inventory::ContainerCreated;
use crate::inventory::{
    Container, ContainerId, ContainerKind, Inventory, InventoryDomain, InventoryError,
};

pub struct ContainerCreation<'operation> {
    pub container: Container,
    domain: &'operation mut InventoryDomain, // containers: &'operation mut Vec<Container>,
                                             // containers_sequence: &'operation mut usize,
}

impl<'operation> ContainerCreation<'operation> {
    pub fn complete(self) -> Vec<Inventory> {
        let events = vec![ContainerCreated {
            id: self.container.id,
        }];
        self.domain.containers_sequence = self.container.id.0;
        self.domain.containers.push(self.container);
        // *self.containers_sequence = self.container.id.0;
        // self.containers.push(self.container);
        events
    }
}

impl InventoryDomain {
    pub fn create_container(
        &mut self,
        kind: Shared<ContainerKind>,
    ) -> Result<ContainerCreation, InventoryError> {
        Ok(ContainerCreation {
            container: Container {
                id: ContainerId(self.containers_sequence + 1),
                kind,
            },
            domain: self, // containers: &mut self.containers,
                          // containers_sequence: &mut self.containers_sequence,
        })
    }
}
