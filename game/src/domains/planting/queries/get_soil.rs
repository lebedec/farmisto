use crate::planting::{PlantingDomain, PlantingError, Soil, SoilId};

impl PlantingDomain {
    pub fn get_soil_mut(&mut self, id: SoilId) -> Result<&mut Soil, PlantingError> {
        self.soils
            .iter_mut()
            .find(|soil| soil.id == id)
            .ok_or(PlantingError::SoilNotFound { id })
    }

    pub fn get_soil(&self, id: SoilId) -> Result<&Soil, PlantingError> {
        self.soils
            .iter()
            .find(|soil| soil.id == id)
            .ok_or(PlantingError::SoilNotFound { id })
    }
}
