use crate::planting::Planting::PlantUpdated;
use crate::planting::{PlantId, Planting, PlantingDomain, PlantingError};

impl PlantingDomain {
    pub fn water_plant(
        &mut self,
        id: PlantId,
        amount: f32,
    ) -> Result<impl FnOnce() -> Vec<Planting> + '_, PlantingError> {
        let plant = self.get_plant_mut(id)?;
        let operation = move || {
            plant.thirst -= amount;
            if plant.thirst < 0.0 {
                plant.thirst = 0.0;
            }
            vec![PlantUpdated {
                id,
                impact: plant.impact,
                thirst: plant.thirst,
                hunger: plant.hunger,
                growth: plant.growth,
            }]
        };
        Ok(operation)
    }
}
