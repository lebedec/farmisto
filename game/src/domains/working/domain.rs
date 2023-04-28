use crate::collections::{Sequence, Shared};
use serde::{Deserialize, Serialize};

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
    pub duration: f32,
    pub durability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub usize);

pub struct Device {
    pub id: DeviceId,
    pub kind: Shared<DeviceKind>,
    pub enabled: bool,
    pub broken: bool,
    pub progress: f32,
    pub input: bool,
    pub output: bool,
    pub deprecation: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Working {
    DeviceUpdated {
        device: DeviceId,
        enabled: bool,
        broken: bool,
        progress: f32,
        input: bool,
        output: bool,
        deprecation: f32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkingError {
    DeviceNotFound { id: DeviceId },
}
