use crate::persistence::{Collection, Grouping, Id, Knowledge, Persisted, Shared};

#[derive(Default)]
pub struct PlantingDomain {
    pub lands_knowledge: Knowledge<LandKind>,
    pub lands: Collection<Land>,
    pub plants_knowledge: Knowledge<PlantKind>,
    pub plants: Grouping<Plant, LandId>,
}

#[derive(Persisted)]
pub struct LandKind {
    id: usize,
    name: String,
}

#[derive(Id, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LandId(usize);

#[derive(Persisted)]
pub struct Land {
    id: LandId,
    kind: Shared<LandKind>,
}

#[derive(Persisted)]
pub struct PlantKind {
    pub id: usize,
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
    pub fn load(&mut self, connection: &rusqlite::Connection) {
        self.lands_knowledge.load(connection);
        self.lands.load(connection, &self.lands_knowledge);
        self.plants_knowledge.load(connection);
        self.plants.load(connection, &self.plants_knowledge);
    }

    pub fn update(&mut self, _time: f32) -> Vec<Planting> {
        let mut _events = vec![];
        _events
    }
}
