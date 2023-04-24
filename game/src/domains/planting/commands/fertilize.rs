use crate::math::TileMath;
use crate::planting::{Planting, PlantingDomain, PlantingError, SoilId};

impl PlantingDomain {
    pub fn fertilize(
        &mut self,
        id: SoilId,
        tile: [usize; 2],
        quality: f32,
    ) -> Result<impl FnOnce() -> Vec<Planting> + '_, PlantingError> {
        let soil = self.get_soil_mut(id)?;
        let place = tile.fit(soil.kind.width);
        if place >= soil.fertility.len() {
            return Err(PlantingError::OutOfSoil { id, tile });
        }
        let command = move || {
            let fertility = soil.fertility[place];
            soil.fertility[place] = (fertility + quality).min(1.0);
            vec![]
        };
        Ok(command)
    }
}
