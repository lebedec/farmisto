use crate::collections::Shared;
use crate::planting::{Plant, PlantId, PlantKind, Planting, PlantingDomain, PlantingError, SoilId};

impl PlantingDomain {
    pub fn create_plant<'operation>(
        &'operation mut self,
        soil: SoilId,
        kind: &Shared<PlantKind>,
        impact: f32,
    ) -> Result<(PlantId, impl FnOnce() -> Vec<Planting> + 'operation), PlantingError> {
        let id = PlantId(self.plants_sequence + 1);
        let plant = Plant {
            id,
            kind: kind.clone(),
            soil,
            impact,
            thirst: 0.0,
            hunger: 0.0,
            health: 1.0,
            growth: 0.0,
            fruits: kind.max_fruits,
        };
        let operation = move || {
            let events = vec![];
            self.plants_sequence += 1;
            self.plants[soil.0].push(plant);
            events
        };
        Ok((id, operation))
    }
}
