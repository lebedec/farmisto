use std::collections::HashMap;

use crate::collections::Shared;

pub const MAX_LANDS: usize = 128;

pub struct PlantingDomain {
    pub known_lands: HashMap<LandKey, Shared<LandKind>>,
    pub known_plants: HashMap<PlantKey, Shared<PlantKind>>,
    pub lands: Vec<Land>,
    pub lands_sequence: usize,
    pub plants: Vec<Vec<Plant>>,
    pub plants_sequence: usize,
}

impl Default for PlantingDomain {
    fn default() -> Self {
        Self {
            known_lands: Default::default(),
            known_plants: Default::default(),
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
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Planting {
    LandChanged {
        land: LandId,
        map: Vec<Vec<[f32; 2]>>,
    },
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
                land: land.id,
                map: land.map.clone(),
            })
        }
        events
    }
}
