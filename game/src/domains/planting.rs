use std::collections::HashMap;

use crate::collections::Shared;

pub const MAX_LANDS: usize = 128;

pub struct PlantingDomain {
    pub known_lands: HashMap<LandKey, Shared<LandKind>>,
    pub known_plants: HashMap<PlantKey, Shared<PlantKind>>,
    pub lands: Vec<Land>,
    pub plants: Vec<Vec<Plant>>,
}

impl Default for PlantingDomain {
    fn default() -> Self {
        Self {
            known_lands: Default::default(),
            known_plants: Default::default(),
            lands: vec![],
            plants: vec![vec![]; MAX_LANDS],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandKey(pub usize);

pub struct LandKind {
    pub id: LandKey,
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandId(pub usize);

pub type Cell = [f32; 2];

pub struct Land {
    pub id: LandId,
    pub kind: Shared<LandKind>,
    pub map: Vec<Vec<Cell>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantKey(pub usize);

pub struct PlantKind {
    pub id: PlantKey,
    pub name: String,
    pub growth: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantId(pub usize);

#[derive(Clone)]
pub struct Plant {
    pub id: PlantId,
    pub kind: Shared<PlantKind>,
    pub land: LandId,
}

pub enum Planting {
    LandChanged { id: LandId, map: Vec<Vec<Cell>> },
}

impl PlantingDomain {
    pub fn get_land(&self, id: LandId) -> Option<&Land> {
        self.lands.iter().find(|land| land.id == id)
    }

    pub fn update(&mut self, time: f32) -> Vec<Planting> {
        let mut events = vec![];
        for land in self.lands.iter_mut() {
            for row in land.map.iter_mut() {
                for cell in row.iter_mut() {
                    let [capacity, moisture] = *cell;
                    *cell = [capacity, (moisture - 0.1 * time).max(0.0)];
                }
            }
            events.push(Planting::LandChanged {
                id: land.id,
                map: land.map.clone(),
            })
        }
        events
    }
}
