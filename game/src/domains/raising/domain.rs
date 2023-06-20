use serde::{Deserialize, Serialize};

use crate::collections::Shared;

#[derive(Default)]
pub struct RaisingDomain {
    pub herds: Vec<Herd>,
    pub herdsmans: Vec<Herdsman>,
    pub animals_id: usize,
    pub animals: Vec<Animal>,
    pub dead_animals: Vec<Animal>,
    pub tethers_id: usize,
    pub tethers: Vec<Tether>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HerdId(pub(crate) usize);

pub struct Herd {
    pub id: HerdId,
    pub herdsman: HerdsmanId,
}

pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimalKey(pub(crate) usize);

pub struct AnimalKind {
    pub id: AnimalKey,
    pub name: String,
    pub hunger_speed: f32,
    pub thirst_speed: f32,
    pub hunger_damage: f32,
    pub thirst_damage: f32,
    pub death_threshold: f32,
    pub voracity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Behaviour {
    Idle,
    Eating,
    Sleeping,
    Walking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimalId(pub usize);

pub struct Animal {
    pub id: AnimalId,
    pub kind: Shared<AnimalKind>,
    // pub flock: HerdId,
    pub age: f32,
    pub weight: f32,
    // pub sex: Sex,
    pub thirst: f32,
    pub hunger: f32,
    pub voracity: f32,

    pub health: f32,
    pub stress: f32,

    pub behaviour: Behaviour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TetherId(pub usize);

pub struct Tether {
    pub id: TetherId,
    pub animal: Option<AnimalId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HerdsmanId(pub(crate) usize);

pub struct Herdsman {
    pub id: HerdsmanId,
    pub leadership: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Raising {
    AnimalChanged {
        id: AnimalId,
        hunger: f32,
        thirst: f32,
        age: f32,
        weight: f32,
    },
    AnimalHealthChanged {
        id: AnimalId,
        health: f32,
    },
    LeadershipChanged {
        id: HerdsmanId,
        leadership: f32,
    },
    HerdsmanChanged {
        herd: HerdId,
        herdsman: HerdsmanId,
    },
    BehaviourChanged {
        id: AnimalId,
        behaviour: Behaviour,
    },
    BehaviourTriggered {
        id: AnimalId,
        trigger: Behaviour,
        behaviour: Behaviour,
    },
    AnimalTied {
        id: AnimalId,
        tether: TetherId,
    },
    AnimalUntied {
        id: AnimalId,
        tether: TetherId,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RaisingError {
    AnimalNotFound { id: AnimalId },
    TetherNotFound { id: TetherId },
    HerdsmanNotFound { id: HerdsmanId },
}
