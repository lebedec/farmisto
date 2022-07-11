use crate::Storage;
use datamap::{Collection, Grouping, Id, Known, Persisted, Shared};

#[derive(Default)]
pub struct PlantingDomain {
    pub known_lands: Known<LandKind>,
    pub known_plants: Known<PlantKind>,
    pub lands: Collection<Land>,
    pub plants: Grouping<Plant, LandId>,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandKey(usize);

#[derive(Persisted)]
pub struct LandKind {
    id: LandKey,
    name: String,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandId(usize);

#[derive(Persisted)]
pub struct Land {
    id: LandId,
    kind: Shared<LandKind>,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantKey(usize);

#[derive(Persisted)]
pub struct PlantKind {
    pub id: PlantKey,
    pub name: String,
    pub growth: f32,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantId(usize);

#[derive(Persisted)]
pub struct Plant {
    pub id: PlantId,
    pub kind: Shared<PlantKind>,
    #[group]
    pub land: LandId,
}

pub enum Planting {}

impl PlantingDomain {
    pub fn load(&mut self, storage: &Storage) {
        self.known_lands.load(storage);
        self.lands.load(storage, &self.known_lands);
        self.known_plants.load(storage);
        self.plants.load(storage, &self.known_plants);
    }

    pub fn update(&mut self, _time: f32) -> Vec<Planting> {
        let mut _events = vec![];
        _events
    }
}
