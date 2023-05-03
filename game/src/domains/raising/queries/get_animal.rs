use crate::raising::{Animal, AnimalId, RaisingDomain, RaisingError};

impl RaisingDomain {
    pub fn get_animal_mut(&mut self, id: AnimalId) -> Result<&mut Animal, RaisingError> {
        self.animals
            .iter_mut()
            .find(|animal| animal.id == id)
            .ok_or(RaisingError::AnimalNotFound { id })
    }

    pub fn get_animal(&self, id: AnimalId) -> Result<&Animal, RaisingError> {
        self.animals
            .iter()
            .find(|animal| animal.id == id)
            .ok_or(RaisingError::AnimalNotFound { id })
    }
}
