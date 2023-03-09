use crate::collections::Shared;

pub const MAX_SOILS: usize = 128;

pub struct PlantingDomain {
    pub soils: Vec<Soil>,
    pub soils_sequence: usize,
    pub plants: Vec<Vec<Plant>>,
    pub plants_sequence: usize,
}

impl Default for PlantingDomain {
    fn default() -> Self {
        Self {
            soils: vec![],
            soils_sequence: 0,
            plants: vec![vec![]; MAX_SOILS],
            plants_sequence: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoilKey(pub(crate) usize);

pub struct SoilKind {
    pub id: SoilKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SoilId(pub usize);

pub struct Soil {
    pub id: SoilId,
    pub kind: Shared<SoilKind>,
    pub map: Vec<Vec<[f32; 2]>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantKey(pub(crate) usize);

pub struct PlantKind {
    pub id: PlantKey,
    pub name: String,
    pub growth: f32,
    pub flexibility: f32,
    pub transpiration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct PlantId(pub(crate) usize);

#[derive(Clone)]
pub struct Plant {
    pub id: PlantId,
    pub kind: Shared<PlantKind>,
    pub soil: SoilId,
    pub impact: f32,
    pub thirst: f32,
    pub hunger: f32,
    pub health: f32,
    pub growth: f32,
    pub fruits: u8,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Planting {
    SoilChanged {
        soil: SoilId,
        map: Vec<Vec<[f32; 2]>>,
    },
    PlantUpdated {
        id: PlantId,
        impact: f32,
        thirst: f32,
        hunger: f32,
        growth: f32,
    },
    PlantHarvested {
        id: PlantId,
        fruits: u8,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PlantingError {
    PlantNotFound { id: PlantId },
    NotReadyToHarvest { id: PlantId },
    HasNoFruitsToHarvest { id: PlantId },
}

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

    pub fn get_soil(&self, id: SoilId) -> Option<&Soil> {
        self.soils.iter().find(|soil| soil.id == id)
    }

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

    pub fn integrate_impact(&mut self, id: PlantId, impact: f32) -> Result<(), PlantingError> {
        let plant = self.get_plant_mut(id)?;
        plant.impact += impact;
        if plant.impact < -1.0 {
            plant.impact = -1.0;
        }
        if plant.impact > 1.0 {
            plant.impact = 1.0;
        }
        Ok(())
    }
}
