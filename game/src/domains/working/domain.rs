use crate::collections::{Sequence, Shared};

#[derive(Default)]
pub struct WorkingDomain {
    pub devices_id: Sequence,
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceKey(pub usize);

pub struct DeviceKind {
    pub id: DeviceKey,
    pub name: String,
    pub process: Process,
    pub duration: f32,
    pub durability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct DeviceId(pub usize);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode, serde::Deserialize,
)]
pub enum DeviceMode {
    Running,
    Pending,
    Stopped,
    Broken,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode, serde::Deserialize,
)]
pub enum Process {
    CementGeneration,
}

pub struct Device {
    pub id: DeviceId,
    pub kind: Shared<DeviceKind>,
    pub mode: DeviceMode,
    pub resource: u8,
    pub progress: f32,
    pub deprecation: f32,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Working {
    ProcessCompleted {
        device: DeviceId,
        process: Process,
    },
    DeviceUpdated {
        device: DeviceId,
        mode: DeviceMode,
        resource: u8,
        progress: f32,
        deprecation: f32,
    },
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum WorkingError {
    DeviceNotFound { id: DeviceId },
}
