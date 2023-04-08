use crate::collections::{Shared, TemporaryRef};
use crate::working::{
    Device, DeviceId, DeviceKind, DeviceMode, Working, WorkingDomain, WorkingError,
};

impl WorkingDomain {
    pub fn create_device(
        &mut self,
        id: DeviceId,
        kind: &Shared<DeviceKind>,
    ) -> Result<impl FnOnce() -> Vec<Working>, WorkingError> {
        let device = Device {
            id,
            kind: kind.clone(),
            mode: DeviceMode::Pending,
            resource: 0,
            progress: 0.0,
            deprecation: 0.0,
        };
        let mut domain = TemporaryRef::from(self);
        let command = move || {
            domain.devices_id.register(id.0);
            domain.devices.push(device);
            vec![]
        };
        Ok(command)
    }
}
