use crate::collections::Shared;

#[derive(Default)]
pub struct RaisingDomain {
    pub herds: Vec<Herd>,
    pub herdsmans: Vec<Herdsman>,
    pub animals_sequence: usize,
    pub animals: Vec<Animal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct HerdId(pub(crate) usize);

pub struct Herd {
    pub id: HerdId,
    pub herdsman: HerdsmanId,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimalKey(pub(crate) usize);

pub struct AnimalKind {
    pub id: AnimalKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct AnimalId(pub(crate) usize);

pub enum Sex {
    Male,
    Female,
}

pub struct Animal {
    pub id: AnimalId,
    pub kind: Shared<AnimalKind>,
    // pub flock: HerdId,
    pub age: f32,
    // pub sex: Sex,
    pub thirst: f32,
    pub hunger: f32,
    pub health: f32,
    pub stress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct HerdsmanId(pub(crate) usize);

pub struct Herdsman {
    pub id: HerdsmanId,
    pub leadership: f32,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Raising {
    AnimalChanged { id: AnimalId, hunger: f32 },
    LeadershipChanged { id: HerdsmanId, leadership: f32 },
    HerdsmanChanged { herd: HerdId, herdsman: HerdsmanId },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum RaisingError {
    AnimalNotFound { id: AnimalId },
    HerdsmanNotFound { id: HerdsmanId },
}

impl RaisingDomain {
    pub fn load_animals(&mut self, animals: Vec<Animal>, sequence: usize) {
        self.animals_sequence = sequence;
        for animal in animals {
            self.animals.push(animal);
        }
    }

    pub fn get_animal_mut(&mut self, id: AnimalId) -> Result<&mut Animal, RaisingError> {
        self.animals
            .iter_mut()
            .find(|animal| animal.id == id)
            .ok_or(RaisingError::AnimalNotFound { id })
    }

    pub fn get_animal(&self, id: AnimalId) -> Result<&Animal, RaisingError> {
        self.animals
            .iter()
            .find(|animal| animal.id == id)
            .ok_or(RaisingError::AnimalNotFound { id })
    }
}