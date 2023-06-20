use crate::planting::{Plant, PlantingDomain, Soil};

impl PlantingDomain {
    pub fn load_soils(&mut self, soils: Vec<Soil>, sequence: usize) {
        self.soils_sequence = sequence;
        self.soils.extend(soils);
    }

    pub fn load_plants(&mut self, plants: Vec<Plant>, sequence: usize) {
        self.plants_sequence = sequence;
        for plant in plants {
            self.plants[plant.soil.0].push(plant);
        }
    }
}
