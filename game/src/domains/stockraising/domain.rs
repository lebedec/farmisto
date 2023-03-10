use crate::collections::Shared;

pub struct StockraisingDomain {
    pub herds: Vec<Herd>,
    pub herdsmans: Vec<Herdsman>,
    pub livestocks: Vec<Livestock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct HerdId(pub(crate) usize);

pub struct Herd {
    pub id: HerdId,
    pub herdsman: HerdsmanId,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LivestockKey(pub(crate) usize);

pub struct LivestockKind {
    pub key: LivestockKey,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct LivestockId(pub(crate) usize);

pub enum Sex {
    Male,
    Female,
}

pub enum Behaviour {
    Idle,
    EatFood,
    Resting,
    Sleeping,
    Follows,
    Straying,
    Fleeing,
    Kicking,
}

pub struct Livestock {
    pub id: LivestockId,
    pub kind: Shared<LivestockKind>,
    pub flock: HerdId,
    pub age: f32,
    pub sex: Sex,
    pub thirst: f32,
    pub hunger: f32,
    pub health: f32,
    pub stress: f32,
    pub behaviour: Behaviour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct HerdsmanId(pub(crate) usize);

pub struct Herdsman {
    pub id: HerdsmanId,
    pub leadership: f32,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Stockraising {
    LeadershipChanged { id: HerdsmanId, leadership: f32 },
    HerdsmanChanged { herd: HerdId, herdsman: HerdsmanId },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum StockraisingError {
    HerdsmanNotFound { id: HerdsmanId },
}
