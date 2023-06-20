use crate::collections::Shared;
use crate::planting::{Planting, PlantingDomain, PlantingError, Soil, SoilId, SoilKind};

impl PlantingDomain {
    pub fn create_soil(
        &mut self,
        kind: &Shared<SoilKind>,
    ) -> Result<(SoilId, impl FnOnce() -> Vec<Planting> + '_), PlantingError> {
        let id = SoilId(self.soils_sequence + 1);
        let soil = Soil {
            id,
            kind: kind.clone(),
            fertility: vec![0.0; 128 * 128],
        };
        let operation = move || {
            let events = vec![];
            self.soils_sequence += 1;
            self.soils.push(soil);
            events
        };
        Ok((id, operation))
    }
}
