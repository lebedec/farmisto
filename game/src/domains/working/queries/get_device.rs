use crate::working::{Device, DeviceId, WorkingDomain, WorkingError};

impl WorkingDomain {
    pub fn get_device_mut(&mut self, id: DeviceId) -> Result<&mut Device, WorkingError> {
        self.devices
            .iter_mut()
            .find(|device| device.id == id)
            .ok_or(WorkingError::DeviceNotFound { id })
    }

    pub fn get_device(&self, id: DeviceId) -> Result<&Device, WorkingError> {
        self.devices
            .iter()
            .find(|device| device.id == id)
            .ok_or(WorkingError::DeviceNotFound { id })
    }
}
