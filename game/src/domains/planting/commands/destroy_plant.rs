use crate::planting::{PlantId, Planting, PlantingDomain, PlantingError};

impl PlantingDomain {
    pub fn destroy_plant(
        &mut self,
        id: PlantId,
    ) -> Result<(f32, impl FnOnce() -> Vec<Planting> + '_), PlantingError> {
        let plant = self.get_plant(id)?;
        let soil = plant.soil;
        let residue = plant.growth * plant.health;
        let command = move || {
            let index = self.plants[soil.0]
                .iter()
                .position(|plant| plant.id == id)
                .unwrap();
            let _plant = self.plants[soil.0].remove(index);
            vec![]
        };
        Ok((residue, command))
    }
}
