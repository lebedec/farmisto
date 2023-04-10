use crate::working::Working::DeviceUpdated;
use crate::working::{DeviceId, DeviceMode, Working, WorkingDomain, WorkingError};

impl WorkingDomain {
    pub fn toggle_device(
        &mut self,
        id: DeviceId,
    ) -> Result<impl FnOnce() -> Vec<Working> + '_, WorkingError> {
        let device = self.get_device_mut(id)?;
        let command = move || {
            if device.mode == DeviceMode::Stopped {
                device.mode = DeviceMode::Running;
            } else if device.mode == DeviceMode::Running || device.mode == DeviceMode::Pending {
                device.mode = DeviceMode::Stopped
            }
            vec![DeviceUpdated {
                device: device.id,
                mode: device.mode,
                resource: device.resource,
                progress: device.progress,
                deprecation: device.deprecation,
            }]
        };
        Ok(command)
    }
}
