use crate::collections::Shared;

pub const MAX_LANDS: usize = 128;

pub struct PlantingDomain {
    pub lands: Vec<Land>,
    pub lands_sequence: usize,
    pub plants: Vec<Vec<Plant>>,
    pub plants_sequence: usize,
}

impl Default for PlantingDomain {
    fn default() -> Self {
        Self {
            lands: vec![],
            lands_sequence: 0,
            plants: vec![vec![]; MAX_LANDS],
            plants_sequence: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandKey(pub(crate) usize);

pub struct LandKind {
    pub id: LandKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct LandId(pub usize);

pub struct Land {
    pub id: LandId,
    pub kind: Shared<LandKind>,
    pub map: Vec<Vec<[f32; 2]>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantKey(pub(crate) usize);

pub struct PlantKind {
    pub id: PlantKey,
    pub name: String,
    pub growth: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct PlantId(pub(crate) usize);

#[derive(Clone)]
pub struct Plant {
    pub id: PlantId,
    pub kind: Shared<PlantKind>,
    pub land: LandId,
    pub impact: f32,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Planting {
    LandChanged {
        land: LandId,
        map: Vec<Vec<[f32; 2]>>,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PlantingError {
    PlantNotFound { id: PlantId },
}

impl PlantingDomain {
    pub fn load_lands(&mut self, lands: Vec<Land>, sequence: usize) {
        self.lands_sequence = sequence;
        self.lands.extend(lands);
    }

    pub fn load_plants(&mut self, plants: Vec<Plant>, sequence: usize) {
        self.plants_sequence = sequence;
        for plant in plants {
            self.plants[plant.land.0].push(plant);
        }
    }

    pub fn get_land(&self, id: LandId) -> Option<&Land> {
        self.lands.iter().find(|land| land.id == id)
    }

    pub fn get_plant(&self, id: PlantId) -> Result<&Plant, PlantingError> {
        for plants in &self.plants {
            if let Some(plant) = plants.iter().find(|plant| plant.id == id) {
                return Ok(plant);
            }
        }
        Err(PlantingError::PlantNotFound { id })
    }
}
