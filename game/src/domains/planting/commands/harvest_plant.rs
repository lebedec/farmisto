use crate::planting::PlantId;
use crate::planting::Planting;
use crate::planting::Planting::PlantFruitsChanged;
use crate::planting::PlantingDomain;
use crate::planting::PlantingError;
use crate::planting::PlantingError::{HasNoFruitsToHarvest, NotReadyToHarvest};

impl PlantingDomain {
    pub fn harvest_plant(
        &mut self,
        id: PlantId,
        amount: u8,
    ) -> Result<(u8, impl FnOnce() -> Vec<Planting> + '_), PlantingError> {
        let plant = self.get_plant_mut(id)?;
        if plant.growth < 3.0 || plant.growth >= 4.0 {
            return Err(NotReadyToHarvest { id });
        }
        let fruits = (amount as f32).min(plant.fruits).floor();
        if fruits < 1.0 {
            return Err(HasNoFruitsToHarvest { id });
        }
        let operation = move || {
            plant.fruits -= fruits;
            vec![PlantFruitsChanged {
                id,
                fruits: plant.fruits,
            }]
        };
        Ok((fruits as u8, operation))
    }
}
