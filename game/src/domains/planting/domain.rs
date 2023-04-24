use crate::collections::Shared;
use crate::math::Rect;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoilKey(pub(crate) usize);

pub struct SoilKind {
    pub id: SoilKey,
    pub name: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct SoilId(pub usize);

pub struct Soil {
    pub id: SoilId,
    pub kind: Shared<SoilKind>,
    pub fertility: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    PlantUpdated {
        id: PlantId,
        impact: f32,
        thirst: f32,
        hunger: f32,
        growth: f32,
    },
    PlantDamaged {
        id: PlantId,
        health: f32,
    },
    PlantHarvested {
        id: PlantId,
        fruits: u8,
    },
    SoilFertilityInspected {
        soil: SoilId,
        rect: Rect,
        fertility: Vec<f32>,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum PlantingError {
    PlantNotFound { id: PlantId },
    NotReadyToHarvest { id: PlantId },
    HasNoFruitsToHarvest { id: PlantId },
    SoilNotFound { id: SoilId },
    OutOfSoil { id: SoilId, tile: [usize; 2] },
}
