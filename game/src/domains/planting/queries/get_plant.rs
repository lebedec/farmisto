use crate::planting::{Plant, PlantId, PlantingDomain, PlantingError};

impl PlantingDomain {
    pub fn get_plant(&self, id: PlantId) -> Result<&Plant, PlantingError> {
        for plants in &self.plants {
            if let Some(plant) = plants.iter().find(|plant| plant.id == id) {
                return Ok(plant);
            }
        }
        Err(PlantingError::PlantNotFound { id })
    }

    pub fn get_plant_mut(&mut self, id: PlantId) -> Result<&mut Plant, PlantingError> {
        for plants in &mut self.plants {
            if let Some(plant) = plants.iter_mut().find(|plant| plant.id == id) {
                return Ok(plant);
            }
        }
        Err(PlantingError::PlantNotFound { id })
    }
}
