use crate::collections::Shared;
use crate::planting::{LandId, Plant, PlantId, PlantKind, Planting, PlantingDomain, PlantingError};

impl PlantingDomain {
    pub fn create_plant<'operation>(
        &'operation mut self,
        land: LandId,
        kind: Shared<PlantKind>,
        impact: f32,
    ) -> Result<(PlantId, impl FnOnce() -> Vec<Planting> + 'operation), PlantingError> {
        let id = PlantId(self.plants_sequence + 1);
        let plant = Plant {
            id,
            kind,
            land,
            impact,
            thirst: 0.0,
        };
        let operation = move || {
            let events = vec![];
            self.plants_sequence += 1;
            self.plants[land.0].push(plant);
            events
        };
        Ok((id, operation))
    }
}
