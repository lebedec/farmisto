use crate::collections::Shared;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformKey(pub usize);

pub struct PlatformKind {
    pub id: PlatformKey,
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformId(pub usize);

pub struct Platform {
    pub id: PlatformId,
    pub kind: Shared<PlatformKind>,
}

#[derive(Default)]
pub struct BuildingDomain {
    pub known_platforms: HashMap<PlatformKey, Shared<PlatformKind>>,
    pub platforms: Vec<Platform>,
}

pub enum Building {}

impl BuildingDomain {}
