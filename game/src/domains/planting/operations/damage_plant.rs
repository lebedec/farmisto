use crate::planting::PlantId;
use crate::planting::Planting;
use crate::planting::Planting::PlantDamaged;
use crate::planting::PlantingDomain;
use crate::planting::PlantingError;

impl PlantingDomain {
    pub fn damage_plant<'operation>(
        &'operation mut self,
        id: PlantId,
        damage: f32,
    ) -> Result<impl FnOnce() -> Vec<Planting> + 'operation, PlantingError> {
        let plant = self.get_plant_mut(id)?;
        let operation = move || {
            plant.health = (plant.health - damage).max(0.0);
            vec![PlantDamaged {
                id,
                health: plant.health,
            }]
        };
        Ok(operation)
    }
}
